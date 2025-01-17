use super::{GossipUi, Page};
use eframe::egui;
use egui::{Context, Ui};

//pub mod desc;
//pub mod infotriple; TODO: incomment when ready
//pub mod infograph;
pub mod mybutton;

//TODO: WIP We want to move towards the new way Gossip has moved into mod logic into sub folders.
//When there is time, streamline with the people page mod file as used as outcommented examples
//below...
pub(super) fn enter_page(app: &mut GossipUi) {
/*    if app.page == page::peoplelists {
        // nothing yet
    } else if let page::peoplelist(plist) = app.page {
        list::enter_page(app, plist);
    } else if matches!(app.page, page::person(_)) {
        // nothing yet
    }
*//*
    match app.page {
            Page::Desc => {
                self.open_menu(ctx, SubMenu::Infospace);
            }  
            Page::InfoTriple => {
//                self.open_menu(ctx, SubMenu::Infospace);
                infotriple::enter_page();
            }
            Page::InfoGraph => {

                self.open_menu(ctx, SubMenu::Infospace);
            }
            Page::MyButton(FeedKind::Person(pubkey)) => {
                GLOBALS.feed.set_feed_to_person(pubkey.to_owned());
                self.open_menu(ctx, SubMenu::Infospace);
            }
    }*/
}

pub(super) fn update(app: &mut GossipUi, ctx: &Context, _frame: &mut eframe::Frame, ui: &mut Ui) {
    /*if app.page == Page::PeopleLists {
        lists::update(app, ctx, _frame, ui);
    } else if let Page::PeopleList(plist) = app.page {
        list::update(app, ctx, _frame, ui, plist);
    } else if matches!(app.page, Page::Person(_)) {
        person::update(app, ctx, _frame, ui);
    }*/
}
