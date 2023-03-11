use multimap::MultiMap;
use btreemultimap::BTreeMultiMap;
use egui::{Key, Modifiers};
// conf_gen::define_conf!(Conf, "conf_prototype.yaml");

// let c = Conf::load("conf.yaml");
// Conf::save("conf.yaml");

// c.player.jump_dur




macro_rules! define_conf {
    (
        $StructName:ident,
        $($member_name:ident : {
                $member_kind:ident,
                $member_description:expr,
                $member_default:expr $(,)?
        }),* $(,)?
    )=> {
        #[derive(Clone, serde::Serialize, serde::Deserialize)]
        pub struct $StructName {
            $(pub $member_name : $member_kind),*
        }
        
        impl $StructName {
            pub fn save(&self, to:&std::path::Path)-> Result<(), std::io::Error> {
                // names with spaces are a bit nicer 
                // let ym = serde_yaml::Mapping::from_iter([$((
                //     serde_yaml::to_value(stringify!($member_name).replace("_", " ")).unwrap(),
                //     serde_yaml::to_value(&self.$member_name).unwrap()
                // )),*].into_iter());
                std::fs::write(to, serde_yaml::to_string(&self).unwrap())
            }
            pub fn load(from:&std::path::Path)-> Result<Self, Box<dyn std::error::Error>> {
                let ry:serde_yaml::Value = serde_yaml::from_reader(std::io::BufReader::new(std::fs::File::open(from)?))?;
                //reason for the extranious "Ok" is that coercion only happens when there's a ?
                Ok(<Self as serde::Deserialize>::deserialize(ry)?)
            }
            pub fn register_edit_commands<AppState>(ar: &mut crate::weftui::CommandRegistry, adaptor: fn(&mut AppState)-> &mut $StructName){
                $(
                    ar.register_command(crate::weftui::Command{
                        name: format!("change {}", stringify!($member_name).replace("_", " ")),
                        description: $member_description.into(),
                        keybind: None,
                        // action: move |v:&mut AppState|{ &mut adaptor(v).$member_name ... }
                    })
                );+
            }
        }
        impl Default for $StructName {
            fn default()-> Self {
                Self {
                    $($member_name : $member_default),*
                }
            }
        }
    }
}
pub (crate) use define_conf;


//standardized for the config format
fn key_event_to_weftui_event(k:&Key, m:&Modifiers)-> String {
    let mut ret = String::new();
    let consider_punctuating = |chain: &mut String, nxt| { if chain.len() > 0 { chain.push_str("+"); } chain.push_str(nxt); };
    if m.ctrl { consider_punctuating(&mut ret, "ctrl"); }
    if m.alt { consider_punctuating(&mut ret, "alt"); }
    if m.shift { consider_punctuating(&mut ret, "shift"); }
    if m.command || m.mac_cmd { consider_punctuating(&mut ret, "super"); }
    consider_punctuating(&mut ret, k.name());
    ret
}

//at the start of a tweaking action, this is the proportion by which one "thumb" of tweak changes the value. So, for large values, it moves fast, for small values, it moves slow.
const FLOAT_CONF_DYNAMIC_SCALE:f64 = 0.5;
const FLOAT_CONF_DYNAIMC_SCALE_FOR_ZERO:f64 = 1.0;

struct FloatConf{
    name: String,
    description: String,
    v: f64,
    upper_bound: Option<f64>,
    lower_bound: Option<f64>,
    //if None, scale is per FLOAT_CONF_DYNAMIC_SCALE
    scale: Option<f64>,
}

pub struct Command {
    pub name: String,
    pub description: String,
    pub keybind: Option<String>,
    // action: Box<dyn Fn(&mut AppState)>
}


pub struct CommandRegistry{
    commands: Vec<Command>,
    by_name: BTreeMultiMap<String, usize>,
    ///later ones override earlier ones
    by_keybind: MultiMap<String, usize>,
}
impl CommandRegistry {
    pub fn register_command(&mut self, v:Command){
        let ni = self.commands.len();
        self.by_name.insert(v.name.clone(), ni);
        if let Some(ref kb) = v.keybind {
            self.by_keybind.insert(kb.clone(), ni);
        }
        self.commands.push(v);
    }
}



