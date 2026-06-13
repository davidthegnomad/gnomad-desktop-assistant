use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation};

use gnomad_core::chat::{
    format_bytes, remove_staged_attachments, stage_chat_attachments, ChatAttachment, MAX_FILES,
};
use gnomad_core::config::paths::DataPaths;

#[derive(Clone)]
pub struct AttachmentTray {
    pub root: GtkBox,
    chips: GtkBox,
    pending: Rc<RefCell<Vec<ChatAttachment>>>,
    paths: DataPaths,
}

impl AttachmentTray {
    pub fn new(paths: DataPaths) -> Self {
        let root = GtkBox::new(Orientation::Vertical, 4);
        root.add_css_class("attachment-tray");
        let chips = GtkBox::new(Orientation::Horizontal, 6);
        chips.add_css_class("attachment-chips");
        root.append(&chips);
        root.set_visible(false);
        Self {
            root,
            chips,
            pending: Rc::new(RefCell::new(Vec::new())),
            paths,
        }
    }

    pub fn clear(&self) {
        let paths: Vec<String> = self
            .pending
            .borrow()
            .iter()
            .map(|a| a.path.clone())
            .collect();
        remove_staged_attachments(&paths);
        self.pending.borrow_mut().clear();
        self.rebuild_chips();
    }

    pub fn pending(&self) -> Vec<ChatAttachment> {
        self.pending.borrow().clone()
    }

    pub fn pick_files(&self, parent: &impl IsA<gtk4::Widget>) {
        let Some(win) = parent
            .root()
            .and_then(|r| r.downcast::<gtk4::Window>().ok())
        else {
            return;
        };

        let dialog = gtk4::FileDialog::new();
        dialog.set_title("Attach files");
        dialog.set_modal(true);

        let tray = self.clone();
        dialog.open_multiple(
            Some(&win),
            None::<&gtk4::gio::Cancellable>,
            move |result| {
                if let Ok(model) = result {
                    let mut source_paths = Vec::new();
                    let n = model.n_items();
                    for i in 0..n {
                        if let Some(obj) = model.item(i) {
                            if let Ok(file) = obj.downcast::<gtk4::gio::File>() {
                                if let Some(path) = file.path() {
                                    source_paths.push(path.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                    tray.stage_paths(source_paths);
                }
            },
        );
    }

    fn stage_paths(&self, source_paths: Vec<String>) {
        if source_paths.is_empty() {
            return;
        }
        let current = self.pending.borrow().len();
        if current >= MAX_FILES {
            return;
        }
        let allowed = MAX_FILES - current;
        let mut paths = source_paths;
        paths.truncate(allowed);
        match stage_chat_attachments(&self.paths, &paths) {
            Ok(staged) => {
                self.pending.borrow_mut().extend(staged);
                self.rebuild_chips();
            }
            Err(e) => {
                glib::g_warning!("gnomad-attachments", "{e}");
            }
        }
    }

    fn rebuild_chips(&self) {
        while let Some(child) = self.chips.first_child() {
            self.chips.remove(&child);
        }
        let pending = self.pending.borrow();
        if pending.is_empty() {
            self.root.set_visible(false);
            return;
        }
        self.root.set_visible(true);
        for att in pending.iter() {
            self.chips.append(&attachment_chip(att, Rc::clone(&self.pending), self.clone()));
        }
    }

}

fn attachment_chip(
    att: &ChatAttachment,
    pending: Rc<RefCell<Vec<ChatAttachment>>>,
    tray: AttachmentTray,
) -> GtkBox {
    let chip = GtkBox::new(Orientation::Horizontal, 4);
    chip.add_css_class("attachment-chip");

    let label = Label::new(Some(&format!(
        "{} ({})",
        att.name,
        format_bytes(att.size_bytes)
    )));
    label.set_ellipsize(gtk4::pango::EllipsizeMode::Middle);
    label.set_max_width_chars(24);
    chip.append(&label);

    let remove = Button::from_icon_name("window-close-symbolic");
    remove.add_css_class("flat");
    remove.add_css_class("attachment-remove");
    let id = att.id.clone();
    remove.connect_clicked(move |_| {
        let path = pending
            .borrow()
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.path.clone());
        if let Some(path) = path {
            remove_staged_attachments(&[path]);
        }
        pending.borrow_mut().retain(|a| a.id != id);
        tray.rebuild_chips();
    });
    chip.append(&remove);

    chip
}
