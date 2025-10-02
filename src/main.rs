//use slint::Image;
//use slint::Model;
//use slint::Rgba8Pixel;
//use slint::SharedPixelBuffer;

slint::include_modules!();
use anyhow::Result;
use fits_rs::{HDUData, FITS};
use std::cell::RefCell;
use std::rc::Rc;

type FITSHandle = Rc<RefCell<Option<FITS>>>;

fn show_errors(e: &anyhow::Error) -> String {
    let mut output = format!("Error: {}", e);
    for cause in e.chain().skip(1) {
        output.push_str(&format!("\n  caused by: {}", cause));
    }
    output
}

fn main() -> Result<()> {
    // Setup FITS file handle
    let thefits: FITSHandle = Rc::new(RefCell::new(None));

    let ui = AppWindow::new()?;

    let fits = thefits.clone();

    ui.global::<Utils>()
        .on_framenum_array(move |_n: i32| -> slint::ModelRc<slint::SharedString> {
            let options = match fits.borrow().as_ref() {
                Some(f) => (0..f.len())
                    .map(|index| {
                        let hdu_type = match f.at(index).unwrap().data {
                            HDUData::Image(_) => "Image",
                            HDUData::Table(_) => "Table",
                            HDUData::BinTable(_) => "Binary Table",
                            HDUData::None => "None",
                        };
                        slint::SharedString::from(format!("{} : {}", index + 1, hdu_type))
                    })
                    .collect::<Vec<_>>(),
                None => vec![slint::SharedString::from("Err")],
            };

            slint::ModelRc::new(slint::VecModel::from_slice(&options))
        });

    ui.global::<Utils>()
        .on_str2int(|s: slint::SharedString| -> i32 { s.parse::<i32>().unwrap_or(1) });
    ui.global::<Utils>()
        .on_range(|n: i32| -> slint::ModelRc<i32> {
            slint::ModelRc::new(slint::VecModel::from_slice(&(0..n).collect::<Vec<_>>()))
        });
    let fits = thefits.clone();
    ui.global::<Utils>()
        .on_frame_menu_labels(move || -> slint::ModelRc<MenuLabel> {
            let options = match fits.borrow().as_ref() {
                Some(f) => (0..f.len())
                    .map(|index| {
                        let hdu_type = match f.at(index).unwrap().data {
                            HDUData::Image(_) => "Image",
                            HDUData::Table(_) => "Table",
                            HDUData::BinTable(_) => "Binary Table",
                            HDUData::None => "Primary",
                        };
                        MenuLabel {
                            val: slint::SharedString::from(format!("{}", index + 1)),
                            label: slint::SharedString::from(hdu_type),
                        }
                    })
                    .collect::<Vec<_>>(),
                None => vec![MenuLabel {
                    val: slint::SharedString::from("1"),
                    label: slint::SharedString::from("Err"),
                }],
            };

            slint::ModelRc::new(slint::VecModel::from_slice(&options))
        });

    let fits = thefits.clone();
    let weakui = ui.as_weak();
    ui.on_open_file_dialog(move || {
        let file = rfd::FileDialog::new()
            .add_filter("FITS", &["fits", "fit", "fts"])
            .set_title("Open FITS file")
            .pick_file();
        if let Some(file) = file {
            let path = file.as_path().to_string_lossy().to_string();
            let ui = weakui.upgrade().unwrap();

            match FITS::from_file(&path) {
                Ok(f) => {
                    ui.set_filename(path.clone().into());
                    ui.set_numframes(f.len() as i32);
                    ui.set_current_frame(1);
                    fits.borrow_mut().replace(f);
                }
                Err(e) => {
                    ui.invoke_show_warning(show_errors(&e).into());
                    fits.borrow_mut().take();
                }
            };
            println!("Selected file: {}", path);
        }
        true
    });

    // Setup warning dialog
    let wd = WarningDialog::new()?;
    let wdc = wd.as_weak();
    wd.on_hide(move || {
        wdc.upgrade().unwrap().hide().unwrap();
    });
    ui.on_show_warning(move |text: slint::SharedString| {
        wd.set_text(text);
        wd.show().unwrap();
    });

    let weakui = ui.as_weak();
    let fits = thefits.clone();
    ui.on_update_frame(move |new_frame: i32| {
        let _ui = weakui.upgrade().unwrap();
        if let Some(fits) = fits.borrow().as_ref() {
            if let HDUData::Image(img) = &fits.at(new_frame as usize - 1).unwrap().data {
                if img.ndims() == 2 {}
            }
        }
        println!("frame changed to {}", new_frame);
    });

    println!("Starting UI");
    ui.run()?;
    println!("leaving UI");
    Ok(())
}
