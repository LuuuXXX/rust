use crate::core::builder::{Builder, RunConfig, ShouldRun, Step};
use crate::core::config::TargetSelection;
use crate::Compiler;
use crate::utils::tarball::GeneratedTarball;
use crate::utils::tarball::OverlayKind;
use crate::utils::tarball::Tarball;
use crate::core::build_steps::others_tool;

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
pub struct CargoAudit {
    pub compiler: Compiler,
    pub target: TargetSelection,
}

impl Step for CargoAudit {
    type Output = Option<GeneratedTarball>;
    const DEFAULT: bool = true;
    const ONLY_HOSTS: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.alias("cargo-audit")
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(CargoAudit {
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

        let cargo_audit = builder.ensure(others_tool::CargoAudit { compiler, target });

        let mut tarball = Tarball::new(builder, "cargo-audit", &target.triple);
        
        tarball.set_overlay(OverlayKind::CargoAudit);
        tarball.add_file(cargo_audit, "bin", 0o755);
        tarball.add_legal_and_readme_to("share/doc/cargo-audit");
        Some(tarball.generate())
    }
}