use std::path::PathBuf;
use std::process::Command;

use crate::core::build_steps::compile;
use crate::core::build_steps::tool;
use crate::core::build_steps::toolstate::ToolState;
use crate::core::builder;
use crate::core::builder::{Builder, Cargo as CargoCommand, RunConfig, ShouldRun, Step};
use crate::core::config::TargetSelection;
use crate::Compiler;
use crate::Mode;
use crate::utils::exec::BootstrapCommand;
use crate::GitInfo;
use crate::utils::helpers::exe;
use crate::Kind;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ToolBuild {
    compiler: Compiler,
    target: TargetSelection,
    tool: &'static str,
    path: &'static str,
    mode: Mode,
    source_type: tool::SourceType,
    extra_features: Vec<String>,
    /// Nightly-only features that are allowed (comma-separated list).
    allow_features: &'static str,
}

impl Step for ToolBuild {
    type Output = PathBuf;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.never()
    }

    /// Builds a tool in `src/tools`
    ///
    /// This will build the specified tool with the specified `host` compiler in
    /// `stage` into the normal cargo output directory.
    fn run(self, builder: &Builder<'_>) -> PathBuf {
        let compiler = self.compiler;
        let target = self.target;
        let mut tool = self.tool;
        let path = self.path;

        match self.mode {
            Mode::ToolRustc => {
                builder.ensure(compile::Std::new(compiler, compiler.host));
                builder.ensure(compile::Rustc::new(compiler, target));
            }
            Mode::ToolStd => builder.ensure(compile::Std::new(compiler, target)),
            Mode::ToolBootstrap => {} // uses downloaded stage0 compiler libs
            _ => panic!("unexpected Mode for tool build"),
        }

        let mut cargo = prepare_tool_cargo(
            builder,
            compiler,
            self.mode,
            target,
            "build",
            path,
            self.source_type,
            &self.extra_features,
        );
        if !self.allow_features.is_empty() {
            cargo.allow_features(self.allow_features);
        }
        let _guard = builder.msg_tool(
            Kind::Build,
            self.mode,
            self.tool,
            self.compiler.stage,
            &self.compiler.host,
            &self.target,
        );

        let mut cargo = Command::from(cargo);
        // we check this below
        let build_success = builder.run_cmd(BootstrapCommand::from(&mut cargo).allow_failure());

        builder.save_toolstate(
            tool,
            if build_success { ToolState::TestFail } else { ToolState::BuildFail },
        );

        if !build_success {
            crate::exit!(1);
        } else {
            // HACK(#82501): on Windows, the tools directory gets added to PATH when running tests, and
            // compiletest confuses HTML tidy with the in-tree tidy. Name the in-tree tidy something
            // different so the problem doesn't come up.
            if tool == "tidy" {
                tool = "rust-tidy";
            }
            let cargo_out = builder.cargo_out(compiler, self.mode, target).join(exe(tool, target));
            let bin = builder.tools_dir(compiler).join(exe(tool, target));
            builder.copy_link(&cargo_out, &bin);
            bin
        }
    }
}

#[allow(clippy::too_many_arguments)] // FIXME: reduce the number of args and remove this.
pub fn prepare_tool_cargo(
    builder: &Builder<'_>,
    compiler: Compiler,
    mode: Mode,
    target: TargetSelection,
    command: &'static str,
    path: &str,
    source_type: tool::SourceType,
    extra_features: &[String],
) -> CargoCommand {
    let mut cargo = builder::Cargo::new(builder, compiler, mode, source_type, target, command);

    let dir = builder.src.join(path);
    cargo.arg("--manifest-path").arg(dir.join("Cargo.toml"));

    let features = extra_features.to_vec();

    if path.ends_with("cargo-audit") {
        cargo.arg("--locked");
    }

    // clippy tests need to know about the stage sysroot. Set them consistently while building to
    // avoid rebuilding when running tests.
    cargo.env("SYSROOT", builder.sysroot(compiler));

    // if tools are using lzma we want to force the build script to build its
    // own copy
    cargo.env("LZMA_API_STATIC", "1");

    // CFG_RELEASE is needed by rustfmt (and possibly other tools) which
    // import rustc-ap-rustc_attr which requires this to be set for the
    // `#[cfg(version(...))]` attribute.
    cargo.env("CFG_RELEASE", builder.rust_release());
    cargo.env("CFG_RELEASE_CHANNEL", &builder.config.channel);
    cargo.env("CFG_VERSION", builder.rust_version());
    cargo.env("CFG_RELEASE_NUM", &builder.version);
    cargo.env("DOC_RUST_LANG_ORG_CHANNEL", builder.doc_rust_lang_org_channel());
    if let Some(ref ver_date) = builder.rust_info().commit_date() {
        cargo.env("CFG_VER_DATE", ver_date);
    }
    if let Some(ref ver_hash) = builder.rust_info().sha() {
        cargo.env("CFG_VER_HASH", ver_hash);
    }

    let info = GitInfo::new(builder.config.omit_git_hash, &dir);
    if let Some(sha) = info.sha() {
        cargo.env("CFG_COMMIT_HASH", sha);
    }
    if let Some(sha_short) = info.sha_short() {
        cargo.env("CFG_SHORT_COMMIT_HASH", sha_short);
    }
    if let Some(date) = info.commit_date() {
        cargo.env("CFG_COMMIT_DATE", date);
    }
    if !features.is_empty() {
        cargo.arg("--features").arg(&features.join(", "));
    }

    // Enable internal lints for clippy and rustdoc
    // NOTE: this doesn't enable lints for any other tools unless they explicitly add `#![warn(rustc::internal)]`
    // See https://github.com/rust-lang/rust/pull/80573#issuecomment-754010776
    //
    // NOTE: We unconditionally set this here to avoid recompiling tools between `x check $tool`
    // and `x test $tool` executions.
    // See https://github.com/rust-lang/rust/issues/116538
    cargo.rustflag("-Zunstable-options");

    cargo
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CargoLlvmCov {
    pub compiler: Compiler,
    pub target: TargetSelection,
}

impl Step for CargoLlvmCov {
    type Output = PathBuf;
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/cargo-llvm-cov")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(CargoLlvmCov {
            compiler: run.builder.compiler(0, run.builder.config.build),
            target: run.target,
        });
    }

    fn run(self, builder: &Builder<'_>) -> PathBuf {
        builder.ensure(ToolBuild {
            compiler: self.compiler,
            target: self.target,
            tool: "cargo-llvm-cov",
            mode: Mode::ToolRustc,
            path: "src/tools/cargo-llvm-cov",
            source_type: tool::SourceType::Submodule,
            extra_features: Vec::new(),
            allow_features: "",
        })
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Flamegraph {
    pub compiler: Compiler,
    pub target: TargetSelection,
}

impl Step for Flamegraph {
    type Output = PathBuf;
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/flamegraph")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(Flamegraph {
            compiler: run.builder.compiler(0, run.builder.config.build),
            target: run.target,
        });
    }

    fn run(self, builder: &Builder<'_>) -> PathBuf {
        builder.ensure(ToolBuild {
            compiler: self.compiler,
            target: self.target,
            tool: "flamegraph",
            mode: Mode::ToolRustc,
            path: "src/tools/flamegraph",
            source_type: tool::SourceType::Submodule,
            extra_features: Vec::new(),
            allow_features: "",
        })
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CargoFuzz {
    pub compiler: Compiler,
    pub target: TargetSelection,
}

impl Step for CargoFuzz {
    type Output = PathBuf;
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.path("src/tools/cargo-fuzz")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(CargoFuzz {
            compiler: run.builder.compiler(0, run.builder.config.build),
            target: run.target,
        });
    }

    fn run(self, builder: &Builder<'_>) -> PathBuf {
        builder.ensure(ToolBuild {
            compiler: self.compiler,
            target: self.target,
            tool: "cargo-fuzz",
            mode: Mode::ToolRustc,
            path: "src/tools/cargo-fuzz",
            source_type: tool::SourceType::Submodule,
            extra_features: Vec::new(),
            allow_features: "",
        })
    }
}