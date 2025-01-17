use std::collections::HashMap;

use std::collections::BTreeMap;
use bitcoin_hashes::Hash;
use bitcoin_hashes::sha256;
use srf_cat::cat_element_list::CatElementList;
use srf_cat::cat_service_fs::CatServiceFS;
use srf_cat::cat_list::CatList;
use egui::{Align, Context, Layout, RichText, Ui};
use crate::ui::feed::render_a_feed;
use crate::ui::GossipUi;
use crate::ui::FeedKind;
use crate::ui::GLOBALS;
use eframe::{egui, Frame};
use nostr_types::Id;
use egui_winit::egui::Id as Id2;


pub fn update(app: &mut GossipUi, ctx: &Context, _frame: &mut Frame, ui: &mut Ui) {
    
    check_cat_service(app);

    if app.cat_icons_toggled {
        make_icon_picker(app, ui);
    }
    
    //TODO: implement with one of these...
    let mut send_now: bool = false;
    let mut my_send_now: bool = false;

    ui.vertical(
        |ui|{ui.add_space(5.0);}
    );
    ui.heading("Make your own button");

    ui.add_space(12.0);

    ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
        ui.add_space(12.0);
        let reset_label = "Reset fields";

        if ui.button(reset_label).clicked()
        {
            reset_fields(app);
        }
    });

    ui.vertical(
        |ui|{ui.add_space(5.0);}
    );
    
    ui.horizontal(|ui| {
        if ui.button("Choose icon").clicked() {
            app.cat_icons_toggled = !app.cat_icons_toggled;
        }
        ui.label(app.cat_icon.clone());
    });
    
    ui.vertical(
        |ui|{ui.add_space(5.0);}
    );
    
    ui.horizontal(|ui| {
        ui.label("name:");
        ui.add(
            text_edit_line!(app, app.cat_name)
            .hint_text("Type a short description here (preferely one word)")
            .desired_width(f32::INFINITY),
        );
    });
            
    ui.vertical(
        |ui|{ui.add_space(5.0);}
    );

    ui.label("description (optional):");
    ui.add(
        text_edit_multiline!(app, app.cat_desc)
            .id_source("desc_compose_area")
            .hint_text("Type your description of the button here")
            .desired_width(f32::INFINITY)
            .lock_focus(true)
    ,);

    ui.vertical(
        |ui|{ui.add_space(12.0);}
    );

    ui.horizontal(|ui| {
        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
            let send_label = "Create";

            if ui.button(send_label).clicked()
            {
                create_cat(app, ui);
                app.cat_combo.clear();
                app.cat_condi_combo.clear();
            }
        });
    });

//checks is cat combo selection has been updated.
    if app.cat_id_displayed != app.cat_id.clone() {
        show_selected_cat(app, ui);
    }

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);
        
    ui.horizontal(|ui| {
            ui.label("Watch a button: ");

            cat_combo(app, ui);

            if ui.button("Load events").clicked() {
                load_cat_elements(app, ui);//TODO:find a way to load once per selection
            }
            
            if ui.button("Delete selected button").clicked() {
                delete_selected_cat(app);
                reset_fields(app); 
            }
    });
    
    ui.add_space(12.0);
    render_advanced_settings(app, ui);
    ui.separator();
    ui.add_space(12.0);
    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);

    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
        render_elements_selected_cat(app, ctx, ui, app.cat_id.clone());
    });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);

}


//retreive all cat elements for a specific cat (without gui)
fn load_cat_elements(app: &mut GossipUi, ui: &mut Ui) {
    app.cat_list
        .clone()
        .into_iter()
        .for_each(|cat| {
            app.loaded_cat_elements.insert(
                cat.point.clone(),
                <std::option::Option<CatServiceFS> as Clone>::clone(&app.cat_service).unwrap()
                    .get_cat_elements(
                        "main_table".to_string(), 
                        cat.point.clone()
                    )
            );
        });
}

//TODO: display cat list in a bar
fn cat_bar() {}

//display cat list in a combobox
pub(super) fn cat_combo(app: &mut GossipUi, ui: &mut Ui) {

    if !app.has_cat_combo_loaded {
        app.cat_list = <std::option::Option<CatServiceFS> as Clone>::clone(&app.cat_service).unwrap()  
            .get_all_cats_w_conditions("main_table".to_string());
        app.has_cat_combo_loaded = true;
        if app.cat_id != "Select a value" {
            app.cat_combo.insert(app.cat_id.clone(), app.cat_id.clone());
        }
    }

    let cat_id_combo = egui::ComboBox::from_id_source(Id2::from("CatIdCombo"));
    cat_id_combo
        .width(540.0)
        .selected_text(app.cat_id.to_string())
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut app.cat_id,
                "Select value".to_string(),
                "Select value",
            );
            app.cat_list.clone().into_iter().for_each(|x|{
                ui.selectable_value(
                    &mut app.cat_id,
                    x.point.clone(),
                    x.name.clone(),
                );
            });
        });
}

//display cat condition candidate list in a combobox
pub(super) fn cat_condi_combo(app: &mut GossipUi, ui: &mut Ui) {

    if !app.has_cat_combo_loaded {
        app.cat_list = <std::option::Option<CatServiceFS> as Clone>::clone(&app.cat_service).unwrap()  
            .get_all_cats_w_conditions("main_table".to_string());
        app.has_cat_combo_loaded = true;
        if app.cat_condi_id != "Select a value" {
            app.cat_condi_combo.insert(app.cat_condi_id.clone(), app.cat_condi_id.clone());
        }
    }

    let cat_condi_id_combo = egui::ComboBox::from_id_source(Id2::from("CatCondiIdCombo"));
    cat_condi_id_combo
        .width(540.0)
        .selected_text(app.cat_condi_id.to_string())
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut app.cat_condi_id,
                "Select value".to_string(),
                "Select value",
            );
            app.cat_list.clone().into_iter().for_each(|x|{
                ui.selectable_value(
                    &mut app.cat_condi_id,
                    x.point.clone(),
                    x.name.clone(),
                );
            });
        });
}

//display a list of cat elements on a panel.
fn render_elements_selected_cat(app: &mut GossipUi, ctx: &Context, ui: &mut Ui, cat_id: String) {

     let scroll_area = egui::ScrollArea::vertical()
        .id_source("cat_elements_view")
        .auto_shrink([false,false]) //makes not shrink if empty
        .max_height(500.0)
        .min_scrolled_height(500.0)
        .enable_scrolling(true);

     scroll_area
        .show(ui, |ui| {

            if app.loaded_cat_elements.is_empty() || !app.loaded_cat_elements.contains_key(&cat_id) {
                ui.centered_and_justified(|ui|{ 
                    ui.label(RichText::new("<< Cat Elements View >>"));
                });
        
            } else {

                ui.vertical(|ui| {

                    let feed_kind = GLOBALS.feed.get_feed_kind();
                    let load_more = feed_kind.can_load_more();
                    if let FeedKind::Person(pubkey) = feed_kind {
                        let filtered_feed:Vec<String> = app.loaded_cat_elements.clone().get(&cat_id.clone()).unwrap()
                            .get_point_ids();
                        let filtered_feed = filtered_feed
                            .iter()
                            .map(|x|{
                                Id::try_from_hex_string(x).unwrap_or(Id::try_from_hex_string("0000000000000000000000000000000000000000000000000000000000000000").unwrap())
                            })
                            .collect();

                        render_a_feed(
                            app,
                            ctx,
                            ui,
                            filtered_feed,
                            false,
                            &pubkey.as_hex_string(),
                            load_more,
                        ); 
                    }
                });
            }
        });
}


fn render_advanced_settings(app: &mut GossipUi, ui: &mut Ui) {
    
    ui.collapsing("Advanced festures", |ui| {
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);
        
        ui.horizontal(|ui| {
            ui.label("Choose another button for constraint: ");

            cat_condi_combo(app, ui);

            if ui.button("Show after this type").clicked() {
                show_after_this_type(app, app.cat_condi_id.clone(), app.cat_id.clone());
            }
            
            if ui.button("Show this type after").clicked() {
                show_after_this_type(app, app.cat_id.clone(), app.cat_condi_id.clone());
            }
        });
    });
}


fn something_random() -> String{
    let mut r = rand::random::<u64>().to_string();
    let t1 = rand::random::<u8>();
    let t2 = rand::random::<u8>();

    for _n in 1..t1 {
        for _m in 1..t2 {
            let r1 = rand::random::<u64>();
            let r2 = rand::random::<f64>();
            let r3 = rand::random::<i64>();
            r.push_str(format!("{}{}{}",r1,r2,r3).as_str());
        }
    }

    hash_text(r.as_str())
}

fn hash_text(text: &str) -> String {
    let hash_of_string = sha256::Hash::hash(text.as_bytes());
    hash_of_string.to_string()
}


//screen for creating a cat
fn create_cat( app: &mut GossipUi, ui: &mut Ui) {
    <std::option::Option<CatServiceFS> as Clone>::clone(&app.cat_service).unwrap()
        .create_cat(&something_random(), app.cat_name.as_str(), app.cat_icon.as_str(), app.cat_desc.as_str());
    app.has_cat_combo_loaded = false;
}


//like create screen just pre-filled
fn show_selected_cat(app: &mut GossipUi, ui: &mut Ui) {
    let cat = app.cat_list.get(app.cat_id.clone()).unwrap_or_default();
    app.cat_name = cat.name;
    app.cat_icon = cat.icon_uri;
    app.cat_desc = cat.description;
    app.cat_id_displayed = app.cat_id.clone();
}


//TODO::make new cat based on show screen and mark the old desc as deprecated.
fn update_selected_cat() {}


//hit the delete button on the show screen and mark the old desc as deprecated
fn delete_selected_cat(app: &mut GossipUi) {
    <std::option::Option<CatServiceFS> as Clone>::clone(&app.cat_service).unwrap()
        .delete_cat(app.cat_list.get(app.cat_id.clone()).unwrap());
}


pub struct FontBook {
    filter: String,
    font_id: egui::FontId,
    named_chars: BTreeMap<egui::FontFamily, BTreeMap<char, String>>,
}

impl Default for FontBook {
    fn default() -> Self {
        Self {
            filter: Default::default(),
            font_id: egui::FontId::proportional(18.0),
            named_chars: Default::default(),
        }
    }
}

/// Get a map of all available characters mapped with a human readable name if they got one.
fn available_characters(ui: &egui::Ui, family: egui::FontFamily) -> BTreeMap<char, String> {
    ui.fonts(|f| {
        f.lock()
            .fonts
            .font(&egui::FontId::new(10.0, family)) // size is arbitrary for getting the characters
            .characters()
            .iter()
            .filter(|chr| !chr.is_whitespace() && !chr.is_ascii_control())
            .map(|&chr| (chr, char_name(chr)))
            .collect()
    })
}

// Get human readable name for special char or "unknown".
// Helper method for available_characters above...
fn char_name(chr: char) -> String {
    special_char_name(chr)
        .map(|s| s.to_owned())
        .or_else(|| unicode_names2::name(chr).map(|name| name.to_string().to_lowercase()))
        .unwrap_or_else(|| "unknown".to_owned())
}

// Get human readable name for special char if exists hence return type Option.
// Helper method for char_name above...
fn special_char_name(chr: char) -> Option<&'static str> {
    #[allow(clippy::match_same_arms)] // many "flag"
    match chr {
        // Special private-use-area extensions found in `emoji-icon-font.ttf`:
        // Private use area extensions:
        '\u{FE4E5}' => Some("flag japan"),
        '\u{FE4E6}' => Some("flag usa"),
        '\u{FE4E7}' => Some("flag"),
        '\u{FE4E8}' => Some("flag"),
        '\u{FE4E9}' => Some("flag"),
        '\u{FE4EA}' => Some("flag great britain"),
        '\u{FE4EB}' => Some("flag"),
        '\u{FE4EC}' => Some("flag"),
        '\u{FE4ED}' => Some("flag"),
        '\u{FE4EE}' => Some("flag south korea"),
        '\u{FE82C}' => Some("number sign in square"),
        '\u{FE82E}' => Some("digit one in square"),
        '\u{FE82F}' => Some("digit two in square"),
        '\u{FE830}' => Some("digit three in square"),
        '\u{FE831}' => Some("digit four in square"),
        '\u{FE832}' => Some("digit five in square"),
        '\u{FE833}' => Some("digit six in square"),
        '\u{FE834}' => Some("digit seven in square"),
        '\u{FE835}' => Some("digit eight in square"),
        '\u{FE836}' => Some("digit nine in square"),
        '\u{FE837}' => Some("digit zero in square"),

        // Special private-use-area extensions found in `emoji-icon-font.ttf`:
        // Web services / operating systems / browsers
        '\u{E600}' => Some("web-dribbble"),
        '\u{E601}' => Some("web-stackoverflow"),
        '\u{E602}' => Some("web-vimeo"),
        '\u{E603}' => Some("web-twitter"),
        '\u{E604}' => Some("web-facebook"),
        '\u{E605}' => Some("web-googleplus"),
        '\u{E606}' => Some("web-pinterest"),
        '\u{E607}' => Some("web-tumblr"),
        '\u{E608}' => Some("web-linkedin"),
        '\u{E60A}' => Some("web-stumbleupon"),
        '\u{E60B}' => Some("web-lastfm"),
        '\u{E60C}' => Some("web-rdio"),
        '\u{E60D}' => Some("web-spotify"),
        '\u{E60E}' => Some("web-qq"),
        '\u{E60F}' => Some("web-instagram"),
        '\u{E610}' => Some("web-dropbox"),
        '\u{E611}' => Some("web-evernote"),
        '\u{E612}' => Some("web-flattr"),
        '\u{E613}' => Some("web-skype"),
        '\u{E614}' => Some("web-renren"),
        '\u{E615}' => Some("web-sina-weibo"),
        '\u{E616}' => Some("web-paypal"),
        '\u{E617}' => Some("web-picasa"),
        '\u{E618}' => Some("os-android"),
        '\u{E619}' => Some("web-mixi"),
        '\u{E61A}' => Some("web-behance"),
        '\u{E61B}' => Some("web-circles"),
        '\u{E61C}' => Some("web-vk"),
        '\u{E61D}' => Some("web-smashing"),
        '\u{E61E}' => Some("web-forrst"),
        '\u{E61F}' => Some("os-windows"),
        '\u{E620}' => Some("web-flickr"),
        '\u{E621}' => Some("web-picassa"),
        '\u{E622}' => Some("web-deviantart"),
        '\u{E623}' => Some("web-steam"),
        '\u{E624}' => Some("web-github"),
        '\u{E625}' => Some("web-git"),
        '\u{E626}' => Some("web-blogger"),
        '\u{E627}' => Some("web-soundcloud"),
        '\u{E628}' => Some("web-reddit"),
        '\u{E629}' => Some("web-delicious"),
        '\u{E62A}' => Some("browser-chrome"),
        '\u{E62B}' => Some("browser-firefox"),
        '\u{E62C}' => Some("browser-ie"),
        '\u{E62D}' => Some("browser-opera"),
        '\u{E62E}' => Some("browser-safari"),
        '\u{E62F}' => Some("web-google-drive"),
        '\u{E630}' => Some("web-wordpress"),
        '\u{E631}' => Some("web-joomla"),
        '\u{E632}' => Some("lastfm"),
        '\u{E633}' => Some("web-foursquare"),
        '\u{E634}' => Some("web-yelp"),
        '\u{E635}' => Some("web-drupal"),
        '\u{E636}' => Some("youtube"),
        '\u{F189}' => Some("vk"),
        '\u{F1A6}' => Some("digg"),
        '\u{F1CA}' => Some("web-vine"),
        '\u{F8FF}' => Some("os-apple"),

        // Special private-use-area extensions found in `Ubuntu-Light.ttf`
        '\u{F000}' => Some("uniF000"),
        '\u{F001}' => Some("fi"),
        '\u{F002}' => Some("fl"),
        '\u{F506}' => Some("one seventh"),
        '\u{F507}' => Some("two sevenths"),
        '\u{F508}' => Some("three sevenths"),
        '\u{F509}' => Some("four sevenths"),
        '\u{F50A}' => Some("five sevenths"),
        '\u{F50B}' => Some("six sevenths"),
        '\u{F50C}' => Some("one ninth"),
        '\u{F50D}' => Some("two ninths"),
        '\u{F50E}' => Some("four ninths"),
        '\u{F50F}' => Some("five ninths"),
        '\u{F510}' => Some("seven ninths"),
        '\u{F511}' => Some("eight ninths"),
        '\u{F800}' => Some("zero.alt"),
        '\u{F801}' => Some("one.alt"),
        '\u{F802}' => Some("two.alt"),
        '\u{F803}' => Some("three.alt"),
        '\u{F804}' => Some("four.alt"),
        '\u{F805}' => Some("five.alt"),
        '\u{F806}' => Some("six.alt"),
        '\u{F807}' => Some("seven.alt"),
        '\u{F808}' => Some("eight.alt"),
        '\u{F809}' => Some("nine.alt"),
        '\u{F80A}' => Some("zero.sups"),
        '\u{F80B}' => Some("one.sups"),
        '\u{F80C}' => Some("two.sups"),
        '\u{F80D}' => Some("three.sups"),
        '\u{F80E}' => Some("four.sups"),
        '\u{F80F}' => Some("five.sups"),
        '\u{F810}' => Some("six.sups"),
        '\u{F811}' => Some("seven.sups"),
        '\u{F812}' => Some("eight.sups"),
        '\u{F813}' => Some("nine.sups"),
        '\u{F814}' => Some("zero.sinf"),
        '\u{F815}' => Some("one.sinf"),
        '\u{F816}' => Some("two.sinf"),
        '\u{F817}' => Some("three.sinf"),
        '\u{F818}' => Some("four.sinf"),
        '\u{F819}' => Some("five.sinf"),
        '\u{F81A}' => Some("six.sinf"),
        '\u{F81B}' => Some("seven.sinf"),
        '\u{F81C}' => Some("eight.sinf"),
        '\u{F81D}' => Some("nine.sinf"),
        //Easter eggs
        '\u{1F331}' => Some("basil"),
//        '\u{1F919}' => Some("pura vida"),
        '\u{2600}' => Some("GM"),

        _ => None,
    }
}

fn make_icon_picker(app: &mut GossipUi, ui: &mut Ui){

        egui::introspection::font_id_ui(ui, &mut app.fb1.font_id);

        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.add(egui::TextEdit::singleline(&mut app.fb1.filter).desired_width(120.0));
            app.fb1.filter = app.fb1.filter.to_lowercase();
            if ui.button("ｘ").clicked() {
                app.fb1.filter.clear();
            }
        });

        let filter = &app.fb1.filter;
        let named_chars = app.fb1
            .named_chars
            .entry(app.fb1.font_id.family.clone())
            .or_insert_with(|| available_characters(ui, app.fb1.font_id.family.clone()));

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::Vec2::splat(2.0);

                for (&chr, name) in named_chars {
                    if filter.is_empty() || name.contains(filter) || *filter == chr.to_string() {
                        let button = egui::Button::new(
                            egui::RichText::new(chr.to_string()).font(app.fb1.font_id.clone()),
                        )
                        .frame(false);

                        let tooltip_ui = |ui: &mut egui::Ui| {
                            ui.label(
                                egui::RichText::new(chr.to_string()).font(app.fb1.font_id.clone()),
                            );
                            ui.label(format!("{}\nU+{:X}\n\nClick to copy", name, chr as u32));
                        };

                        if ui.add(button).on_hover_ui(tooltip_ui).clicked() {
                            app.cat_icon = chr.to_string();
                            app.cat_icons_toggled = false;
                        }
                    }
                }
            });
        });
    }


fn reset_fields(app: &mut GossipUi) {
    //cat button space:
    app.cat_combo= HashMap::new();
    app.cat_condi_combo= HashMap::new();
    app.cat_id= "Select a value".to_owned();
    app.cat_condi_id= "Select a value".to_owned();
    app.loaded_cat_elements= HashMap::new();
    app.cat_name= String::new();
    app.cat_icon= String::new(); 
    app.cat_desc= String::new();
    //new cat fields
    app.cat_list= CatList::new();
    app.cat_selected= Option::None;
    app.cat_elements= CatElementList::new();
    app.has_cat_combo_loaded= false;
}

fn check_cat_service(app: &mut GossipUi) {
    
        let feed_kind = GLOBALS.feed.get_feed_kind();
        //TODO:: load_more should be used in the future.
        let _load_more = feed_kind.can_load_more();
        if let FeedKind::Person(pubkey) = feed_kind {
            
            if app.cat_service.is_none() || <std::option::Option<CatServiceFS> as Clone>::clone(&app.cat_service).unwrap().space_id != pubkey.as_hex_string() {
                app.cat_service = Some(CatServiceFS::new(pubkey.as_hex_string()));
            }
        }
}

fn show_after_this_type(app: &mut GossipUi, super_cat_id: String, sub_cat_id: String) {

    let super_cat_triple_id = app.cat_list.get(super_cat_id)
    .clone()
    .unwrap()
    .point;

    let sub_cat_triple_id = app.cat_list.get(sub_cat_id)
    .clone()
    .unwrap()
    .triple_id;

    <std::option::Option<CatServiceFS> as Clone>::clone(&app.cat_service).unwrap()
        .create_cat_condi(super_cat_triple_id, sub_cat_triple_id);
    
    //cleaning up to get feed to auto reload new conditions.
    reset_fields(app);
    app.cat_list = <std::option::Option<CatServiceFS> as Clone>::clone(&app.cat_service).unwrap()  
        .get_all_cats_w_conditions("main_table".to_string());

}
