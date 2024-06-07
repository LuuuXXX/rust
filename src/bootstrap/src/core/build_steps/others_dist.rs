use crate::core::builder::{Builder, Kind, RunConfig, ShouldRun, Step};
use crate::core::config::TargetSelection;
use crate::Compiler;
use crate::utils::tarball::GeneratedTarball;
use crate::utils::tarball::OverlayKind;
use crate::utils::tarball::Tarball;
use crate::core::build_steps::others_tool;

use std::collections::HashSet;
use crate::core::build_steps::dist::*;

#[derive(Debug, PartialOrd, Ord, Clone, Hash, PartialEq, Eq)]
pub struct CargoLlvmCov {
    pub compiler: Compiler,
    pub target: TargetSelection,
}

impl Step for CargoLlvmCov {
    type Output = Option<GeneratedTarball>;
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.alias("cargo-llvm-cov")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(CargoLlvmCov {
            compiler: run.builder.compiler_for(
                run.builder.top_stage,
                run.builder.config.build,
                run.target,
            ),
            target: run.target,
        });
    }

    fn run(self, builder: &Builder<'_>) -> Option<GeneratedTarball> {
        let compiler = self.compiler;
        let target = self.target;

        let cargo_llvm_cov = builder.ensure(others_tool::CargoLlvmCov { compiler, target });

        let mut tarball = Tarball::new(builder, "cargo-llvm-cov", &target.triple);
        
        tarball.set_overlay(OverlayKind::CargoLlvmCov);
        tarball.add_file(cargo_llvm_cov, "bin", 0o755);
        tarball.add_legal_and_readme_to("share/doc/cargo-llvm-cov");
        Some(tarball.generate())
    }
}

#[derive(Debug, PartialOrd, Ord, Clone, Hash, PartialEq, Eq)]
pub struct Flamegraph {
    pub compiler: Compiler,
    pub target: TargetSelection,
}

impl Step for Flamegraph {
    type Output = Option<GeneratedTarball>;
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.alias("flamegraph")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(Flamegraph {
            compiler: run.builder.compiler_for(
                run.builder.top_stage,
                run.builder.config.build,
                run.target,
            ),
            target: run.target,
        });
    }

    fn run(self, builder: &Builder<'_>) -> Option<GeneratedTarball> {
        let compiler = self.compiler;
        let target = self.target;

        let flamegraph = builder.ensure(others_tool::Flamegraph { compiler, target });

        let mut tarball = Tarball::new(builder, "flamegraph", &target.triple);
        
        tarball.set_overlay(OverlayKind::Flamegraph);
        tarball.add_file(flamegraph, "bin", 0o755);
        tarball.add_legal_and_readme_to("share/doc/flamegraph");
        Some(tarball.generate())
    }
}

#[derive(Debug, PartialOrd, Ord, Clone, Hash, PartialEq, Eq)]
pub struct CargoFuzz {
    pub compiler: Compiler,
    pub target: TargetSelection,
}

impl Step for CargoFuzz {
    type Output = Option<GeneratedTarball>;
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.alias("cargo-fuzz")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(CargoFuzz {
            compiler: run.builder.compiler_for(
                run.builder.top_stage,
                run.builder.config.build,
                run.target,
            ),
            target: run.target,
        });
    }

    fn run(self, builder: &Builder<'_>) -> Option<GeneratedTarball> {
        let compiler = self.compiler;
        let target = self.target;

        let cargo_fuzz = builder.ensure(others_tool::CargoFuzz { compiler, target });

        let mut tarball = Tarball::new(builder, "cargo-fuzz", &target.triple);
        
        tarball.set_overlay(OverlayKind::CargoFuzz);
        tarball.add_file(cargo_fuzz, "bin", 0o755);
        tarball.add_legal_and_readme_to("share/doc/cargo-fuzz");
        Some(tarball.generate())
    }
}

#[derive(Debug, PartialOrd, Ord, Clone, Hash, PartialEq, Eq)]
pub struct XuanWusExtended {
    stage: u32,
    host: TargetSelection,
    target: TargetSelection,
}

impl Step for XuanWusExtended {
    type Output = ();
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        let builder = run.builder;
        run.alias("xuanwu-extended").default_condition(builder.config.extended)
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(XuanWusExtended {
            stage: run.builder.top_stage,
            host: run.builder.config.build,
            target: run.target,
        });
    }

    /// Creates a combined installer for the specified target in the provided stage.
    fn run(self, builder: &Builder<'_>) {
        let target = self.target;
        let stage = self.stage;
        let compiler = builder.compiler_for(self.stage, self.host, self.target);

        builder.info(&format!("Dist xuanwu-extended stage{} ({})", compiler.stage, target));

        let mut tarballs = Vec::new();
        let mut built_tools = HashSet::new();
        macro_rules! add_component {
            ($name:expr => $step:expr) => {
                if let Some(tarball) = builder.ensure_if_default($step, Kind::Dist) {
                    tarballs.push(tarball);
                    built_tools.insert($name);
                }
            };
        }

        // When rust-std package split from rustc, we needed to ensure that during
        // upgrades rustc was upgraded before rust-std. To avoid rustc clobbering
        // the std files during uninstall. To do this ensure that rustc comes
        // before rust-std in the list below.
        tarballs.push(builder.ensure(Rustc { compiler: builder.compiler(stage, target) }));
        tarballs.push(builder.ensure(Std { compiler, target }).expect("missing std"));

        if target.ends_with("windows-gnu") {
            tarballs.push(builder.ensure(Mingw { host: target }).expect("missing mingw"));
        }

        add_component!("rust-docs" => Docs { host: target });
        add_component!("rust-json-docs" => JsonDocs { host: target });
        add_component!("rust-demangler"=> RustDemangler { compiler, target });
        add_component!("cargo" => Cargo { compiler, target });
        add_component!("rustfmt" => Rustfmt { compiler, target });
        add_component!("rls" => Rls { compiler, target });
        add_component!("rust-analyzer" => RustAnalyzer { compiler, target });
        add_component!("llvm-components" => LlvmTools { target });
        add_component!("clippy" => Clippy { compiler, target });
        add_component!("miri" => Miri { compiler, target });
        add_component!("analysis" => Analysis { compiler, target });
        add_component!("rustc-codegen-cranelift" => CodegenBackend {
            compiler: builder.compiler(stage, target),
            backend: "cranelift".to_string(),
        });
        add_component!("llvm-bitcode-linker" => LlvmBitcodeLinker {compiler, target});
        // extral tools
        add_component!("cargo-llvm-cov" => CargoLlvmCov {compiler, target});
        add_component!("cargo-fuzz" => CargoFuzz {compiler, target});
        add_component!("flamegraph" => Flamegraph {compiler, target});

        // Avoid producing tarballs during a dry run.
        if builder.config.dry_run() {
            return;
        }

        let tarball = Tarball::new(builder, "rust-xuanwu", &target.triple);
        let _generated = tarball.combine(&tarballs);

        let mut license = String::new();
        license += &builder.read(&builder.src.join("COPYRIGHT"));
        license += &builder.read(&builder.src.join("LICENSE-APACHE"));
        license += &builder.read(&builder.src.join("LICENSE-MIT"));
        license.push('\n');
        license.push('\n');

        let rtf = r"{\rtf1\ansi\deff0{\fonttbl{\f0\fnil\fcharset0 Arial;}}\nowwrap\fs18";
        let mut rtf = rtf.to_string();
        rtf.push('\n');
        for line in license.lines() {
            rtf.push_str(line);
            rtf.push_str("\\line ");
        }
        rtf.push('}');
    }
}