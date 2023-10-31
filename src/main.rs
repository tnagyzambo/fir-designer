//!   ```math
//!   \mathbf{J} \coloneqq
//!   \def\arraystretch{1.5}
//!   \begin{pmatrix}
//!   \frac{\partial r_1}{\partial x_1} & \cdots & \frac{\partial r_1}{\partial x_n} \\
//!   \frac{\partial r_2}{\partial x_1} & \cdots & \frac{\partial r_2}{\partial x_n} \\
//!   \vdots & \ddots & \vdots \\
//!   \frac{\partial r_m}{\partial x_1} & \cdots & \frac{\partial r_m}{\partial x_n}
//!   \end{pmatrix}.
//!   ```

mod fir;
mod gui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "FIR Filter Designer",
        options,
        Box::new(|cc| Box::new(gui::App::new(cc))),
    )
}
