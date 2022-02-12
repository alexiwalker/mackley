use crate::mackley::application::Application;

mod mackley;

fn main() {
    let mut application = Application::new(8787, None, None);

    application.listen();
}
