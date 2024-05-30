use crate::core::builder::{Builder, Kind, RunConfig, ShouldRun, Step};
use crate::core::config::TargetSelection;
use crate::Compiler;
use crate::utils::tarball::GeneratedTarball;

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

        // let rust_analyzer = builder.ensure(tool::RustAnalyzer { compiler, target });

        let mut tarball = Tarball::new(builder, "cargo-llvm-cov", &target.triple);
        tarball.set_overlay(OverlayKind::RustAnalyzer);
        tarball.is_preview(true);
        tarball.add_file(rust_analyzer, "bin", 0o755);
        // tarball.add_legal_and_readme_to("share/doc/rust-analyzer");
        Some(tarball.generate())
    }
}