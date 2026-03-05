pub mod app;
pub mod config;
pub mod event;
pub mod grpc;
pub mod model;
pub mod ui;

// Optional PyO3 module — only compiled with the "python" feature.
#[cfg(feature = "python")]
mod python_module {
    use pyo3::prelude::*;

    /// Placeholder Python module for future labgrid-tui Python extensions.
    /// Build with: maturin develop --features python
    #[pymodule]
    fn labgrid_tui(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add("__version__", env!("CARGO_PKG_VERSION"))?;
        Ok(())
    }
}
