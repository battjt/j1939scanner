use std::rc::Rc;
use std::sync::{Arc, Mutex};

// yikes. Comment out the next line, then try to make sense of that error message!
use anyhow::*;

mod j1939;
mod j1939da_ui;
mod multiqueue;
#[cfg_attr(not(target_os = "windows"), path = "sim.rs")]
#[cfg_attr(target_os = "windows", path = "rp1210.rs")]
mod rp1210;
mod rp1210_parsing;

use fltk::app::*;
use fltk::group::*;
use fltk::menu::*;
use fltk::prelude::*;
use fltk::window::Window;
use j1939::packet::*;
use j1939da_ui::J1939Table;
use multiqueue::*;
use rp1210::*;

pub fn main() -> Result<()> {
    //create abstract CAN bus
    let bus: MultiQueue<J1939Packet> = MultiQueue::new();

    // log everything
    //bus.log();

    // UI
    create_application(bus.clone())?.run()?;

    Err(anyhow!("Application should not stop running."))
}

fn create_application(bus: MultiQueue<J1939Packet>) -> Result<App> {
    let application = App::default();
    let window = Window::default()
        .with_label("J1939DA Tool - Solid Design")
        .with_size(800, 600);

    let tab = Tabs::new(10, 10, 500 - 20, 450 - 20, "");

    let grp = Group::new(10, 35, 500 - 20, 450 - 45, "J1939DA");

    let j1939_table = Rc::new(Mutex::new(J1939Table::new()));
    j1939da_ui::create_ui(j1939_table.clone());
    grp.end();

    let grp = Group::new(10, 35, 500 - 20, 450 - 45, "J1939DA");
    j1939da_ui::j1939da_log(&bus);
    grp.end();

    let mut menu = menu::SysMenuBar::default().with_size(800, 35);
    menu.set_frame(FrameType::FlatBox);
    menu.add
    menu.add_emit(
        "&File/J1939DA...\t",
        Shortcut::None,
        menu::MenuFlag::Normal,
        *s,
        Message::New,
    );

    let menu = Menu::new();
        menu.append(
            &create_j1939da_menu(&j1939_table, &window).expect("Unable to create J1939 menu"),
        );
        files_item.set_submenu(Some(&menu));
        menubar.append(&files_item);
    }
    {
        let rp1210_menu = MenuItem::with_label("RP1210");
        rp1210_menu.set_submenu(create_rp1210_menu(bus.clone()).ok().as_ref());
        menubar.append(&rp1210_menu);
    }
    let vbox = Box::builder().orientation(Orientation::Vertical).build();
    vbox.pack_start(&menubar, false, false, 0);
    vbox.pack_end(&notebook, true, true, 0);
    window.add(&vbox);
    window.show_all();
    Ok(application)
}

fn create_j1939da_menu(
    j1939_table: &Rc<Mutex<J1939Table>>,
    window: &ApplicationWindow,
) -> Result<MenuItem> {
    let j1939_menu = MenuItem::with_label("J1939DA...");
    let j1939_table = j1939_table.clone();
    j1939_menu.connect_activate(glib::clone!(@weak window => move |_| {
        let file_chooser = gtk::FileChooserDialog::new(
            Some("Open File"),
            Some(&window),
            gtk::FileChooserAction::Open,
        );
        file_chooser.add_buttons(&[
            ("Open", gtk::ResponseType::Ok),
            ("Cancel", gtk::ResponseType::Cancel),
        ]);
        let ff = FileFilter::new();
        ff.add_pattern("*.xlsx");
        file_chooser.add_filter(&ff);
        let j1939_table = j1939_table.clone();
        file_chooser.connect_response( move |file_chooser, response| {
            if response == gtk::ResponseType::Ok {
                let filename = file_chooser.filename().expect("Couldn't get filename");
                let filename = filename.to_str();
                filename.map(|f|{
                    j1939_table.lock().expect("Unable to unlock model.")
                    .file(f).expect("Unable to load J1939DA");
                });
                        }
            file_chooser.close();
        });

        file_chooser.show_all();
    }));
    return Ok(j1939_menu);
}

fn create_rp1210_menu(bus: MultiQueue<J1939Packet>) -> Result<Menu> {
    let rp1210_menu = Menu::new();
    let closer: Arc<Mutex<Option<std::boxed::Box<dyn Fn() -> ()>>>> = Arc::new(Mutex::new(None));
    {
        // Add the close RP1210 option
        let device_menu_item = MenuItem::with_label("Disconnect");
        let c1 = closer.clone();
        device_menu_item.connect_activate(move |_| {
            let mut closer = c1.lock().unwrap();
            // execute close if there is a prior rp1210 adapter
            closer.as_ref().map(|a| a());
            *closer = None;
        });
        rp1210_menu.append(&device_menu_item);
    }
    for product in rp1210_parsing::list_all_products()? {
        let product_menu_item = MenuItem::with_label(&product.description);
        rp1210_menu.append(&product_menu_item);
        let product_menu = Menu::new();

        // Add all RP1210 J1939 devices
        for device in product.devices {
            let device_menu_item = MenuItem::with_label(&device.description);
            let pid = product.id.clone();
            let bus = bus.clone();
            let closer = closer.clone();
            device_menu_item.connect_activate(move |_| {
                let mut closer = closer.lock().unwrap();
                // execute close if there is a prior rp1210 adapter
                closer.as_ref().map(|a| a());
                // create a new adapter
                let rp1210 = Rp1210::new(&pid, bus.clone()).unwrap();
                *closer = Some(rp1210.run(device.id, "J1939:Baud=Auto", 0xF9).ok().unwrap());
            });
            product_menu.add(&device_menu_item);
        }
        product_menu_item.set_submenu(Some(&product_menu));
    }

    Ok(rp1210_menu)
}
