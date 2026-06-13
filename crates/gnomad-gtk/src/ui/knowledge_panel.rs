use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, DropDown, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow};
use libadwaita as adw;
use libadwaita::prelude::AdwWindowExt;

use gnomad_core::config::paths::DataPaths;
use gnomad_core::knowledge::{
    delete_file, import_files, install_skill_pack, list_files_filtered, list_skill_packs,
    open_knowledge_folder, CATEGORIES,
};

struct KnowledgeUi {
    paths: DataPaths,
    category_dd: DropDown,
    import_cat_dd: DropDown,
    file_list: ListBox,
    status_label: Label,
    packs_box: GtkBox,
}

impl KnowledgeUi {
    fn selected_category(&self) -> Option<&'static str> {
        let idx = self.category_dd.selected() as usize;
        if idx == 0 {
            None
        } else {
            CATEGORIES.get(idx - 1).copied()
        }
    }

    fn import_category(&self) -> &'static str {
        CATEGORIES
            .get(self.import_cat_dd.selected() as usize)
            .copied()
            .unwrap_or("uploads")
    }

    fn refresh(&self) {
        let filter = self.selected_category();
        match list_files_filtered(&self.paths, filter) {
            Ok(files) => {
                self.status_label.set_text(&format!("{} file(s)", files.len()));
                while let Some(child) = self.file_list.first_child() {
                    self.file_list.remove(&child);
                }
                if files.is_empty() {
                    let row = ListBoxRow::new();
                    let label = Label::new(Some("No files in this category."));
                    label.add_css_class("dim-label");
                    label.set_margin_top(12);
                    label.set_margin_bottom(12);
                    row.set_child(Some(&label));
                    self.file_list.append(&row);
                } else {
                    for file in files {
                        let row = ListBoxRow::new();
                        let row_box = GtkBox::new(Orientation::Horizontal, 8);
                        row_box.set_margin_top(8);
                        row_box.set_margin_bottom(8);
                        row_box.set_margin_start(8);
                        row_box.set_margin_end(8);

                        let info = GtkBox::new(Orientation::Vertical, 2);
                        info.set_hexpand(true);
                        let name = Label::new(Some(&file.name));
                        name.set_halign(gtk4::Align::Start);
                        name.set_xalign(0.0);
                        name.add_css_class("heading");
                        info.append(&name);
                        let meta = Label::new(Some(&format!(
                            "{} · {} bytes",
                            file.category, file.size_bytes
                        )));
                        meta.set_halign(gtk4::Align::Start);
                        meta.set_xalign(0.0);
                        meta.add_css_class("dim-label");
                        info.append(&meta);
                        row_box.append(&info);

                        let delete_btn = Button::with_label("Delete");
                        let paths = self.paths.clone();
                        let file_id = file.id.clone();
                        let refresh = self.refresh_cb();
                        delete_btn.connect_clicked(move |_| {
                            let _ = delete_file(&paths, &file_id);
                            refresh();
                        });
                        row_box.append(&delete_btn);

                        row.set_child(Some(&row_box));
                        self.file_list.append(&row);
                    }
                }
            }
            Err(err) => self.status_label.set_text(&err),
        }

        while let Some(child) = self.packs_box.first_child() {
            self.packs_box.remove(&child);
        }
        match list_skill_packs() {
            Ok(packs) if packs.is_empty() => {
                let label = Label::new(Some("No skill packs bundled."));
                label.add_css_class("dim-label");
                self.packs_box.append(&label);
            }
            Ok(packs) => {
                for pack in packs {
                    let row = GtkBox::new(Orientation::Horizontal, 8);
                    let info = GtkBox::new(Orientation::Vertical, 2);
                    info.set_hexpand(true);
                    let title = Label::new(Some(&pack.name));
                    title.set_halign(gtk4::Align::Start);
                    title.set_xalign(0.0);
                    title.add_css_class("heading");
                    info.append(&title);
                    let desc = Label::new(Some(&format!(
                        "{} · {} skill(s)",
                        pack.description, pack.skill_count
                    )));
                    desc.set_wrap(true);
                    desc.set_halign(gtk4::Align::Start);
                    desc.set_xalign(0.0);
                    desc.add_css_class("dim-label");
                    info.append(&desc);
                    row.append(&info);

                    let install = Button::with_label("Install");
                    install.add_css_class("suggested-action");
                    let paths = self.paths.clone();
                    let pack_id = pack.id.clone();
                    let refresh = self.refresh_cb();
                    install.connect_clicked(move |_| {
                        let _ = install_skill_pack(&paths, &pack_id);
                        refresh();
                    });
                    row.append(&install);
                    self.packs_box.append(&row);
                }
            }
            Err(err) => {
                let label = Label::new(Some(&err));
                label.set_wrap(true);
                label.add_css_class("dim-label");
                self.packs_box.append(&label);
            }
        }
    }

    fn refresh_cb(&self) -> Rc<dyn Fn()> {
        let ui = self.clone_handles();
        Rc::new(move || ui.refresh())
    }

    fn clone_handles(&self) -> KnowledgeUi {
        KnowledgeUi {
            paths: self.paths.clone(),
            category_dd: self.category_dd.clone(),
            import_cat_dd: self.import_cat_dd.clone(),
            file_list: self.file_list.clone(),
            status_label: self.status_label.clone(),
            packs_box: self.packs_box.clone(),
        }
    }
}

pub fn show_knowledge_panel(parent: Option<&impl IsA<gtk4::Window>>, paths: DataPaths) {
    let win = adw::Window::new();
    win.set_title(Some("Knowledge library"));
    win.set_default_size(560, 640);
    if let Some(p) = parent {
        win.set_transient_for(Some(p));
        win.set_modal(true);
    }

    let root = GtkBox::new(Orientation::Vertical, 0);
    let header = adw::HeaderBar::new();
    let close = Button::with_label("Close");
    {
        let win = win.clone();
        close.connect_clicked(move |_| win.close());
    }
    header.set_title_widget(Some(&Label::new(Some("Knowledge library"))));
    header.pack_end(&close);
    root.append(&header);

    let body = GtkBox::new(Orientation::Vertical, 16);
    body.set_margin_top(16);
    body.set_margin_bottom(16);
    body.set_margin_start(16);
    body.set_margin_end(16);
    body.set_vexpand(true);

    let intro = Label::new(Some(
        "Skills, agent briefs, and reference files are injected into every chat. Import files or install starter packs.",
    ));
    intro.set_wrap(true);
    intro.set_halign(gtk4::Align::Start);
    intro.add_css_class("dim-label");
    body.append(&intro);

    let filter_row = GtkBox::new(Orientation::Horizontal, 8);
    let filter_label = Label::new(Some("Show"));
    filter_row.append(&filter_label);
    let mut cat_labels: Vec<&str> = vec!["All"];
    cat_labels.extend_from_slice(CATEGORIES);
    let category_dd = DropDown::from_strings(&cat_labels);
    filter_row.append(&category_dd);
    let status_label = Label::new(None);
    status_label.set_hexpand(true);
    status_label.set_halign(gtk4::Align::End);
    status_label.add_css_class("dim-label");
    filter_row.append(&status_label);
    body.append(&filter_row);

    let scroll = ScrolledWindow::builder()
        .vexpand(true)
        .min_content_height(240)
        .build();
    let file_list = ListBox::new();
    file_list.add_css_class("boxed-list");
    file_list.set_selection_mode(gtk4::SelectionMode::None);
    scroll.set_child(Some(&file_list));
    body.append(&scroll);

    let import_row = GtkBox::new(Orientation::Horizontal, 8);
    import_row.append(&Label::new(Some("Import to")));
    let import_cat_dd = DropDown::from_strings(CATEGORIES);
    import_row.append(&import_cat_dd);
    let import_btn = Button::with_label("Import files…");
    import_btn.add_css_class("suggested-action");
    import_row.append(&import_btn);
    let open_btn = Button::with_label("Open folder");
    import_row.append(&open_btn);
    body.append(&import_row);

    let packs_title = Label::new(Some("Starter skill packs"));
    packs_title.add_css_class("heading");
    packs_title.set_halign(gtk4::Align::Start);
    body.append(&packs_title);
    let packs_box = GtkBox::new(Orientation::Vertical, 8);
    body.append(&packs_box);

    let ui = Rc::new(RefCell::new(KnowledgeUi {
        paths: paths.clone(),
        category_dd: category_dd.clone(),
        import_cat_dd: import_cat_dd.clone(),
        file_list: file_list.clone(),
        status_label: status_label.clone(),
        packs_box: packs_box.clone(),
    }));

    category_dd.connect_notify_local(Some("selected"), {
        let ui = Rc::clone(&ui);
        move |_, _| ui.borrow().refresh()
    });

    open_btn.connect_clicked({
        let ui = Rc::clone(&ui);
        let paths = paths.clone();
        move |_| {
            let _ = open_knowledge_folder(&paths);
            ui.borrow().refresh();
        }
    });

    import_btn.connect_clicked({
        let ui = Rc::clone(&ui);
        let win = win.clone();
        move |_| {
            let dialog = gtk4::FileDialog::new();
            dialog.set_title("Import knowledge files");
            dialog.set_modal(true);
            let paths = ui.borrow().paths.clone();
            let category = ui.borrow().import_category().to_string();
            let refresh = ui.borrow().refresh_cb();
            dialog.open_multiple(
                Some(&win),
                None::<&gtk4::gio::Cancellable>,
                move |result| {
                    if let Ok(model) = result {
                        let mut sources = Vec::new();
                        let n = model.n_items();
                        for i in 0..n {
                            if let Some(obj) = model.item(i) {
                                if let Ok(file) = obj.downcast::<gtk4::gio::File>() {
                                    if let Some(path) = file.path() {
                                        sources.push(path);
                                    }
                                }
                            }
                        }
                        if !sources.is_empty() {
                            let _ = import_files(&paths, &sources, &category);
                            refresh();
                        }
                    }
                },
            );
        }
    });

    let scrolled_body = ScrolledWindow::builder().vexpand(true).build();
    scrolled_body.set_child(Some(&body));
    root.append(&scrolled_body);
    win.set_content(Some(&root));
    ui.borrow().refresh();
    win.present();
}
