use gtk4::gdk::Texture;
use gtk4::gdk_pixbuf::Pixbuf;

pub struct About;

impl About {
    pub fn create_dialog() -> gtk4::AboutDialog {
        let about = gtk4::AboutDialog::new();
        about.set_program_name("Gosub Browser".into());
        about.set_version(Some("0.0.1"));
        about.set_website(Some("https://www.gosub.io"));
        about.set_website_label("Gosub Website");
        about.set_copyright(Some("© 2024 Gosub Team"));
        about.set_license_type(gtk4::License::MitX11);
        // about.set_logo_icon_name(Some("gosub"));

        if let Ok(logo_pixbuf) = Pixbuf::from_resource_at_scale("/io/gosub/browser-gtk/assets/gosub.svg", 128, 128, true) {
            let logo_texture = Texture::for_pixbuf(&logo_pixbuf);
            about.set_logo(Some(&logo_texture));
        }
        about.set_comments(Some("A simple browser written in Rust and GTK"));

        about.set_authors(&["Gosub Team", "Joshua Thijssen", "SharkTheOne"]);
        about.add_credit_section("Networking", &["Gosub Team"]);
        about.add_credit_section("HTML5 parser", &["Gosub Team"]);
        about.add_credit_section("CSS3 parser", &["Gosub Team"]);
        about.add_credit_section("Renderer", &["Gosub Team"]);
        about.add_credit_section("Javascript engine", &["Gosub Team"]);
        about.add_credit_section("UI", &["Gosub Team"]);
        about.add_credit_section("GTK integration", &["Gosub Team"]);
        about.add_credit_section("Rust integration", &["Gosub Team"]);
        about.set_translator_credits(Some("Gosub Team"));

        about
    }
}
