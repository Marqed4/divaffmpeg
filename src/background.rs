use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::Paragraph,
    Frame,
};

//                      ¡WARNING PRESERVE WHITESPACE!
 pub const ASTOLFO_TOP: &str = r#"
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣤⠶⣶⣤⣔⣶⡶⣦⣤⣠⣶⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⢀⣴⣯⡏⠉⣴⣿⢿⢿⣿⢷⣶⣍⡻⣇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⢠⡶⢤⣴⣿⠟⠀⢁⡞⠛⡟⢣⠐⢳⡀⠈⢻⣯⡺⢦⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⢸⣇⣾⣟⠎⠀⢠⡞⠀⡀⢀⠆⠀⠀⣧⠀⡀⠹⡳⣦⣿⡦⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⡸⣻⣟⡟⠀⠀⢸⠁⢀⢧⢸⡾⡀⠶⣿⡿⣳⣅⣼⢻⠙⢿⡶⢤⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠞⠉⡿⢡⢡⠀⠀⣿⠀⣿⣯⣭⣹⢝⡊⢸⠚⢻⠝⢺⣗⢆⣸⡷⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⢠⢳⣮⣣⣤⣄⡟⣇⡿⣻⡿⣿⠉⠻⣜⠐⠿⣛⡻⠿⠿⠵⡳⣿⣧⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⢀⡿⢻⢡⡿⣼⡃⢡⠈⣴⠈⠒⠋⠀⠀⠐⠀⠀⠀⠌⡟⡦⣌⡉⠻⡉⢣⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⢸⠃⢹⣾⠀⢹⣷⡜⣷⣿⣧⠀⠀⢴⠞⠛⣿⠀⢀⢧⡜⡟⠀⠉⠓⢄⡀⠈⠓⠤⣀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠈⠀⢸⢹⡆⢸⣿⡛⡷⣝⣽⠳⢦⣌⡓⠀⣢⠔⡡⢻⡋⠀⠀⠀⠀⠀⠑⢦⡀⠀⠀⠑⠢⣄⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠈⣼⢇⡼⠿⣿⣿⡿⣿⡄⠀⠀⢹⢉⡰⠛⠁⠈⠁⠀⠀⠀⠀⠀⠀⠀⠙⢶⡀⠀⠀⠀⠙⠢⡀⠀
⠀⠀⠀⠀⠀⢏⣞⠝⠀⠀⠸⣿⣿⡺⣮⣤⢀⣘⠓⠶⣶⡔⢶⣖⢤⡤⠤⠤⠠⣖⢲⣦⡤⠽⢦⡀⠀⠀⠀⠘⣆
⠀⠀⠀  ⣾⡾⠀⠀⠀⢀⠚⣿⣷⡈⠻⣆⡈⠁⢁⣈⣇⠘⣿⣄⠀⠀⠀⠀⢸⠈⡿⡇⠀⠀⠉⠀⠀⠀⠀⢸
⠀⠀⠀ ⣫⠟⡇⠀⠀⠀⠘⠀⠈⠻⣿⣶⣾⣽⣲⣗⠋⢻⣦⣿⠀⠀⠀⠀⠀⢠⣤⣆⡧⠤⠤⠄⠤⠐⠒⠂⠁
⠀⠀  ⢯⡎⠀⠀⠀⠀⠀⣴⠁⠀⠀⠉⠙⢟⠻⠿⣿⣼⡟⣬⠂⣧⠴⠐⠂⠚⠋⠛⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀  ⣿⡖⠒⠂⠀⠒⡳⠀⠀⠀⠀⠀⠈⠑⠚⠓⣟⣉⣳⠀⢱⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
"#;

/*

This is the middle section that we rendered
in specifically on the start page paragraph.

⠀⠀⣿⡧⢺⢻⣿⣿⣓⣲⣾⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⡇⠫⣆⠸⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⡻⡅⠊⣶⡇⠀⠀⠀⢸⡆⠀⠀⠀⠀⠀⠀⠀⠀⢀⡇⠀⠀⠸⡀
*/

//                      ¡WARNING PRESERVE WHITESPACE!
pub const ASTOLFO_BOTTOM: &str = r#"
⠀⠀    ⠈⡇⠀⠀⠀⢸⢇⠀⠀⠀⠀⠀⠀⠀⠀⠘⣴⡀⠀⠀⢸⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀    ⣐⡏⠀⠀⠀⣼⡸⠀⠀⠀⠀⠀⠀⠀⠀⠀⢣⠱⡄⠀⡟⣷⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀   ⣯⠀⠀⠀⡟⡇⠀⠀⠀⠀⠀⣀⣀⡀⠤⠬⢆⡿⣄⣃⡈⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀  ⣿⠀⠀⠀⢱⣷⠈⠉⠉⠉⠉⠀⠀⠀⠀⠀⠘⠁⢸⠧⠞⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠈⠯⣆⠀⢀⡴⠁⠀⠀⠀⠀⠀⠀⠀⠀⢀⠆⠀⢀⡎⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠈⣳⠟⣦⡀⠀⠀⠀⠀⠀⠀⠀⠀⡿⠀⠀⢸⡃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⢠⡾⢇⠀⠀⠉⠓⠲⠤⠤⠤⠤⠤⠤⠤⠲⢲⣻⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⣰⠏⠀⡸⠓⢵⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣨⣼⣇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⢀⣾⠛⢀⡞⠁⠀⠀⡞⠉⠉⠉⠁⡖⠀⠈⠹⡍⠉⠈⠙⣞⣆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⢠⡴⡟⠉⠀⡾⠀⠀⠀⡸⠀⠀⠀⠀⢠⠇⠀⠀⠀⡇⠀⠀⠀⠘⡜⢦⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⢸⣿⡽⠀⠀⡰⠁⠀⠀⢰⠃⠀⠀⠀⠀⣼⠄⠀⠀⠀⢱⡄⠀⠀⠀⠹⣄⢳⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⢨⣿⣄⠀⣴⠃⠀⠀⢀⡏⠀⠀⠀⠀⠀⣯⠀⠀⠀⠀⢸⣳⠀⠀⠀⠀⠙⣦⢝⣶⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠻⢿⣿⣥⣀⡀⠀⡼⠀⠀⠀⠀⠀⠀⣇⠀⠀⠀⠀⠀⡟⢇⠀⢀⠀⣀⣌⣿⡃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠸⠟⠙⢻⣿⣶⣿⣿⣷⣶⣤⣀⣰⡿⣿⣶⣶⣦⣤⣼⣾⠿⠛⠛⠋⠉⠉⢳⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⢧⠉⠙⠁⠈⠉⠛⠛⠚⠛⢀⡇⠉⢯⠙⠉⠁⠀⠀⠀⠀⠀⠀⠀⢣⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
"#;

//                      ¡WARNING PRESERVE WHITESPACE!
pub const HOT_GIRL: &str = r#"
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⡶⠛⠻⡆⠀⡠⢤⠖⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⡀⠀⠀⠀⠀⠀⠀⣀⣰⣿⠀⠀⣼⣧⠎⣠⣎⠤⣤⠆⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⣾⢉⢿⣶⣤⣴⣶⣿⣿⣿⣧⣀⣸⣿⠟⢉⡇⣴⠊⠀⢀⡔⠒⠖⢦⡀⠀⠀
⠀⠀⠀⠀⢹⣷⢀⣾⣿⣿⣿⣿⣿⣿⣧⣄⠈⢹⡰⣽⣿⡟⠓⠈⢹⠂⣴⣾⢙⡧⡀⠀
⠀⠀⠀⠀⠀⢉⣿⡿⢿⣟⣻⣷⣿⣿⣿⣿⣿⡍⠀⣸⣿⣿⣿⣿⣷⣶⠉⣷⡿⠛⢇⠀
⠀⠀⠀⠀⢀⣾⣿⣿⣿⣿⣿⡿⣿⣿⣿⣿⣿⣿⣦⣿⣿⣿⣿⡟⠁⠀⠋⠉⢧⠀⠈⢢
⠀⠀⠤⣤⡾⣿⣿⣿⣿⣿⣿⠗⣒⣛⣻⣿⣿⣿⣿⣿⣿⣿⣿⣄⠄⠀⠀⠀⠈⢇⠀⠀
⠀⠀⠀⠀⣸⣿⣿⣿⣿⢿⡿⠻⠻⡿⢿⣿⣿⣿⠟⠙⣿⣿⣿⣤⡤⠀⠀⠀⠀⠸⡀⠀
⠀⠀⠀⠀⣟⢸⣿⣿⣿⣿⡇⠀⠀⠀⢉⣿⣿⡅⣈⣽⣿⠋⠉⠀⡄⠀⠀⠀⠀⠀⠃⠀
⠀⠀⠀⠀⠈⠀⠻⣿⣷⠻⠀⠀⠀⠀⠸⣿⣟⡳⣿⣿⠻⠦⠀⡰⠁⠀⠀⠀⣀⠠⠖⠀
⠀⠀⠀⠀⠀⣰⣿⣿⡟⠓⢌⡻⠂⠀⢠⡋⣼⡁⢿⢯⣣⠤⠴⠤⠤⠔⠚⠉⠀⠀⠀⠀
⠀⠀⠀⠀⢸⣹⠟⠀⠀⠀⠀⠈⠢⠒⣹⡏⢡⡷⠾⡿⡏⠀⠀⠀⠀⠀⠀⠀⠀⠀⡀⠐
⠀⠀⠀⠀⣷⣟⠁⠀⠀⢀⢔⡾⠋⠉⠟⣝⣃⣧⡼⠁⣧⡴⠀⠀⣀⣠⡤⠖⠂⠁⠀⠀
⠀⠀⠀⠀⠳⣿⣦⡀⣀⣾⣋⣤⠔⢒⢶⢫⠻⣭⡉⢢⡅⢠⣶⡟⠉⠁⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠘⠷⡽⠋⢡⠖⠁⡔⠁⢸⠈⠀⢨⢛⣦⠛⣏⡽⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⢠⣞⠀⡀⠀⠀⡌⠀⠀⠈⠀⡀⠉⠛⠁⠀⠙⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠉⡿⠛⠟⠺⠓⠶⠛⠻⠶⢷⣶⠶⠷⣶⠚⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⢰⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠂⣸⣹⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⡎⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⡽⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠠⠀⢠⡞⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⢻⠀⠀⠀⠀⠀⠀⠀⠀⠘⢿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠸⣧⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⡰⢻⠀⢰⡀⠀⠀⠀⠀⠀⠀⡜⠉⠱⣄⠀⠀⣰⣿⡄⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠒⢤⢮⣀⣠⣧⠀⢷⠀⠀⠀⠀⠀⢠⠇⠀⠀⠈⠢⣤⣿⣿⢷⠀⠀⠀⠀⠀⠀⠀
⠀⠠⢴⣁⡀⠹⡋⢹⡆⠘⠁⠀⠀⠀⠀⢸⠀⠀⠀⠀⣀⣈⣻⣟⣸⠀⠀⠀⠀⠀⠀⠀
⢸⠅⠋⠙⠒⠯⣄⣸⢧⠀⠀⠀⠀⠀⠀⣸⣴⣶⣿⣿⣿⣿⣿⣿⣇⠀⠀⠀⠀⠀⠀⠀
⠚⠲⡀⠀⣄⡙⣶⣿⣾⣷⣶⣶⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣆⠀⠀⠀⠀⠀⠀
⠀⠀⠈⠳⠿⠟⠉⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⣿⣿⣿⣿⡿⠋⠉⠁⠀⠀⠀⠀⠀⢹⣿⠇⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⡏⠳⣌⡉⢉⠟⠁⠀⠀⠀⠀⠀⠀⠀⠀⠈⡜⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠉⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡼⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⡸⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⠞⠀⠀⠀⠀⠀⠀⠀⠀⠀
"#;

//                      ¡WARNING PRESERVE WHITESPACE!
pub const FELIX_ARGYLE: &str = r#"⠀⠀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⠊⢣⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡴⠉⢠⠘⢆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡘⠀⢸⢻⠀⠺⢆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⣆⠑⡢⢄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⠕⠒⠛⠀⣇⠀⠈⢢⢀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⣟⣆⠀⠀⠀⠋⠖⣢⣀⠀⣀⡀⠠⢤⡏⢣⡀⠀⠀⠸⡄⠀⠀⣻⣽⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢈⡾⠈⢦⡀⠀⠀⢀⡠⣼⠟⠁⣄⠀⠀⠁⢀⡟⠀⣀⠀⢳⠀⠀⢸⢸⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⣛⡁⠀⠀⠙⢦⠔⣡⠊⠀⠀⠀⠈⠁⢤⠀⠈⠉⠉⠁⠑⠺⠇⠀⠈⣾⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢹⠉⡏⣠⠋⡴⠁⠀⠀⠀⠀⡇⢸⠈⢧⠀⠀⠀⠀⡟⠒⠤⣀⣀⣀⣱⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠱⡱⠣⢰⠃⡀⠀⠀⠀⢀⡇⡼⡄⠈⢇⠀⠀⠀⠹⡄⠀⢹⡇⠀⢈⠳⡀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠇⢰⢺⠰⡇⠀⠀⠀⣼⣾⠁⠹⡄⠘⡄⠠⠀⠀⢇⣰⠋⣟⢄⣸⠀⢡⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠃⣸⣸⣷⣇⠀⢀⣸⠟⣁⣤⣤⡘⣾⠁⠀⠀⠀⢠⣇⣀⣿⣆⣙⣦⠀⡆⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣧⠀⣿⠇⣌⣟⢦⠏⠁⠈⢿⣿⣿⢸⢸⠀⠀⡀⠀⣾⣿⣸⣿⢻⡣⣻⠿⢾⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡧⢻⣀⢸⣾⣿⣿⠀⠀⠀⠀⠘⠓⠁⠀⡾⢀⡔⠁⠀⠛⢉⡉⠙⠃⠹⠏⢠⢸⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡴⡇⠈⢿⣌⣟⠛⢃⠀⠀⠀⠀⠀⠀⠀⣼⣵⠋⠀⠀⠀⠀⣸⠀⠀⢠⠀⣰⡞⡞⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⣿⡄⡀⠉⢻⠀⠀⢀⡀⢀⡀⠀⠀⠈⠁⡏⠀⠀⠀⠀⣰⡗⠀⢀⡞⣰⣿⡕⠁⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠳⣱⠀⠸⡄⠀⢿⡿⠛⢻⠀⠀⠀⠀⡇⡀⠀⣠⣾⠟⢀⣴⣿⠞⡡⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢹⡼⢰⠙⢦⡈⢧⣀⡼⠀⠀⢀⡠⢾⣧⢾⣿⣤⢾⢯⡴⠙⠶⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⢮⡧⢈⢿⣦⠤⢤⣴⠚⠉⠀⠈⠋⠈⢻⡵⠛⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠚⠁⠳⠼⠿⣇⣀⣡⠤⠶⠲⢮⢇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⢀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣤⠾⠸⠐⣿⣿⡛⠉⠙⠳⢿⡤⣀⣀⣀⠀⠀⠀⠀⠀⠀⠀⠀⣷⠤⣿
⢸⡽⣦⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡠⠤⠒⣾⠋⠥⠤⠬⠤⣧⣀⣧⠤⠤⠤⠄⠘⣯⠉⠉⠙⢷⡄⠀⠀⠀⠙⣄⠈⠙⠽
⠀⢻⡄⠉⠛⠷⣦⣀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠋⠀⠀⠀⣿⠀⠀⠀⠀⢀⡞⣿⣽⣧⡀⠀⠀⠀⣿⠀⠀⠀⠈⣷⠀⠀⠀⠀⠙⠻⣉⠁
⠀⠀⠻⣆⠀⠀⠀⠙⠷⣄⠀⠀⠀⠀⠀⠀⡇⠀⣄⠀⠀⡏⠀⠀⠀⠠⡾⣿⡏⢻⡿⠛⠀⠀⠀⣿⡀⢰⠃⠀⢸⡆⠀⠀⠀⠀⠀⠚⢛
⠀⠀⠀⢹⣄⠀⠀⠀⠄⠘⣷⡀⠀⠀⠀⢀⠇⠀⠈⡆⢀⣧⠔⢑⠦⠤⢄⣬⣡⠤⣥⣰⣋⠵⠯⣌⣇⡇⠀⠀⠀⡇⠀⠀⠀⠀⢀⢿⠈
⠀⠀⠀⠀⢷⠀⠀⠀⠠⠀⠘⣷⡀⠀⠀⢸⠀⠀⠀⢸⡏⡵⠋⠁⠉⠑⢮⡉⣆⣸⡡⠋⠀⠀⠀⠈⢧⡎⠀⠀⠀⢱⠀⠀⠀⠀⣼⠃⠀
⠀⠀⠀⠀⠘⣆⠀⠐⡊⢤⡀⠸⣧⠀⠀⡞⠀⠀⠀⡰⡿⠀⠀⠀⠀⠀⠀⠈⢾⡿⠁⠀⠀⠀⠀⠀⢸⣹⣴⡶⣶⣿⡆⠀⠀⡼⠃⠀⠀
⣠⣤⠀⠀⠀⣽⡆⠀⠉⠸⣆⠀⢿⡄⢠⠧⣀⣀⡀⢱⡇⠀⠀⠀⠀⠀⠀⢀⣸⣇⣀⣀⣀⣀⣀⣀⣼⣯⠴⠾⠋⠙⣿⣀⠎⠀⠀⠀⣰
⠻⣧⡄⠀⠀⠁⠳⡄⠁⠀⠸⣄⢸⢧⡼⢧⣀⣀⢙⡯⣧⣀⣠⡴⠶⢻⠛⠻⢿⣿⡛⠁⢏⠏⠙⠻⡻⣼⠀⠀⠀⠀⣹⠏⠀⠀⠀⢠⠇
⠀⠀⠙⢷⣤⡀⠀⠘⠲⣄⢀⠘⣾⢻⡷⠬⣀⡘⢻⢃⡼⠋⡰⠀⠀⡆⠀⠀⠙⣿⠁⠀⠈⠳⡀⠀⠘⢎⢧⠀⠀⡔⠁⠀⠀⠀⢀⡞⠁
⠀⠀⠀⠀⠙⠿⣦⣄⠀⠠⠉⠉⣹⣄⡹⢷⣀⡈⡹⠋⠀⡜⠁⠀⠀⠃⠀⠀⢰⡏⢧⠀⣠⠴⣳⢀⢴⣾⣫⡇⢸⠁⠀⠀⠀⠀⡜⠁⡀
⠀⠀⠀⠀⠀⠀⠈⠛⢷⡖⠒⠋⠁⡎⠉⠓⠲⢤⠛⢶⣤⣷⠒⠢⠤⠿⠉⠁⣿⣿⣿⠉⠀⠀⠉⠁⢸⠀⠀⠹⡈⠀⠀⠀⣠⠞⠀⢠⠁
⠀⠀⠀⠀⠀⠀⠀⠀⠀⢿⡀⠀⣸⠀⠀⠀⠀⡏⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⢿⢹⢻⠀⠀⠀⠀⠀⢸⠀⠀⠀⠳⣀⣠⠖⠁⠀⢠⡏⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢇⣰⠃⠀⠀⠀⣸⢀⡤⠒⠙⣺⣀⠀⠀⠀⠀⠀⢸⢸⢸⠀⠀⠀⠀⠀⠈⣧⡒⠒⠒⠮⣄⡀⠀⢸⡟⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⡇⠀⠀⠀⠀⣿⡁⠀⠀⢸⠊⠀⠑⢄⠀⠀⠀⣣⣼⣸⠀⠀⠀⠀⠀⡜⠙⢧⠀⠀⠀⢀⡟⠂⠀⣿⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣾⠀⠀⠀⠀⢸⠁⣹⠀⠀⣾⠀⠀⠀⢀⡗⠀⠀⢸⣦⣿⠀⠀⠀⠀⢰⠁⠀⠘⣆⠀⠀⣎⣀⠀⠀⡷⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⡇⠀⠀⠀⢠⠃⢀⣇⡠⢾⢿⠀⢀⡴⠋⠀⠀⠀⢸⢸⢸⠀⠀⠀⠀⠸⣄⡀⠀⢹⠙⢶⡏⠉⠉⠑⠻⠖⠠
⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⠁⠀⠀⢠⡧⠚⠁⠀⣠⠃⠸⢤⠋⠀⠀⠀⠀⠀⢸⢸⠈⡀⠀⠀⠀⠀⠀⠉⢳⣸⠀⠀⠙⢦⠀⠀⢀⠴⠊
⠀⠀⠀⠀⠀⠀⠀⠀⣸⠇⠀⠀⢰⠯⠤⣄⣠⠞⠁⠀⠀⢸⡤⠂⠀⠀⠀⠀⢸⢸⡀⡇⠀⠀⠀⠀⠀⠀⠀⠙⡆⠀⣠⠤⠽⣦⣼⡀⠀
⠀⠀⠀⠀⠀⠀⠀⢠⠟⠀⠀⢀⣞⣀⠤⢺⠷⠒⠒⡄⢀⠏⠀⠀⠀⠀⠀⢀⡎⢻⣏⢹⡀⠀⠀⠀⠀⠀⠀⠀⢱⣰⣃⠀⠀⠀⠉⠓⠂
"#;

//                      ¡WARNING PRESERVE WHITESPACE!
pub const EMO_POP_TOP: &str = r#"
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⠶⣶⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⢠⠐⠦⣆⣄⡀⡠⣄⣴⣲⣢⢵⣞⠀⠀⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠣⡀⠀⣩⣟⡾⣿⣻⣿⣿⡿⢿⡿⣧⣼⡾⡡⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠸⢯⣾⣇⢝⣏⣯⡿⡿⠟⡟⣏⢿⡾⡗⡿⠸⢤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠘⣿⣯⣿⣯⣟⡵⠋⠀⠀⠈⡿⢻⣩⣃⣿⡕⡄⢤⣹⡶⢤⡀⠀⠀⠀⠀⠀⠀
⠀⠀⢀⣿⣿⣏⣿⣷⡇⣴⣚⣉⣉⡉⣷⣟⣶⢿⡇⢱⣖⡋⡴⠚⢻⢶⠀⠀⠀⠀⠀
⠀⠀⠐⣿⣿⡇⡇⢿⡇⠈⠙⠾⠿⠁⣿⣾⣯⣸⡇⠈⠀⠳⠗⣖⠋⣼⢷⡄⠀⠀⠀
⠀⠀⠁⠙⣿⡹⡇⠈⠀⠀⠀⠀⠀⠀⣯⣽⣿⡫⣇⡄⠀⠀⠀⠀⠹⡟⠋⢀⡵⡄⠀
⠀⠀⠀⠀⠟⠈⢯⠀⢋⡷⠂⠀⠀⣟⡟⣯⣧⠋⠟⠀⠀⠀⠀⠀⠀⠀⠙⣏⣀⣀⣧
⠀⠀⠀⠀⠀⠀⠀⠈⠳⠤⠖⣞⠁⠀⢸⡵⠃⠀⢀⣠⣖⣶⠲⠶⠶⠖⠚⠻⠝⠛⡃
⠀⠀⠀⠀⠀⠀⠀⠀⡠⣤⣤⠧⣴⣦⣴⣠⡾⣯⣯⢯⡿⣏⣇⠀⠀⠀⠀⠀⠀⡀⠃
⠀⠀⠀⠀⠀⠀⣰⣿⠟⠉⠀⠀⣶⠀⠀⠀⠻⣟⠶⣾⣿⣯⣿⢿⣴⠤⠖⠉⠀⠀⠀
⠀⠀⠀⠀⠀⡞⠁⠁⠘⢻⢯⣭⣿⡒⠒⠀⠀⠀⠀⠙⠷⢒⠟⠁⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⢠⠃⠀⠀⠀⠰⠗⠋⠉⠓⢓⣀⡀⠀⠀⡀⣠⠎⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⣷⣀⣀⡡⠤⠾⠿⠯⠭⠭⠭⠭⢶⣯⡗⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠉⠛⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡾⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⣸⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
"#;

/*

This is the middle section that we rendered
in specifically on the start page paragraph.

⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠻⡇⠀⠀⠀
*/

//                      ¡WARNING PRESERVE WHITESPACE!
pub const EMO_POP_BOTTOM: &str = r#"
⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠻⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⣴⣇⠀⢦⠀⠀⠀⠀⠀⠀⠀⢹⣤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⡜⣿⡿⠀⠸⣄⠀⠀⠀⠀⠀⠀⠀⢿⠙⠦⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⢰⡏⠛⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⡀⠤⠓⠒⠐⠚⠳⢤⡀⠀⠀⠀⠀⠀⠀
⠀⡜⢉⠆⠀⣿⣇⡀⠠⠀⠔⠐⠛⠊⠈⠀⠀⠀⠀⠓⢄⡀⠀⠀⠈⠳⢄⡀⠀⠀⠀
⠀⠉⠻⢷⡧⠽⡏⠀⠀⠀⠀⠀⠀⠀⠰⡄⠀⠀⠀⠀⠀⠀⠈⠒⠀⢀⠖⠁⠀⠀⠀
⠀⠀⠀⠀⠀⣇⠀⠀⠀⠸⠁⠀⠀⠀⠀⠀⠹⡄⠀⠀⠀⠀⠀⠀⡴⠃⠀⠀⠀⠀⠀
⠀⠀⠀⣀⢉⠂⠀⠀⠀⠆⠀⠀⠀⠀⠀⠀⠀⠀⠣⡄⠀⠀⢀⡾⠁⠀⠀⠀⠀⠀⠀
⠀⠀⡠⢁⡏⠀⠀⢠⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⠤⠋⣴⠁⠀⠀⠀⠀⠀⠀⠀
⠀⣠⠃⡜⠀⠀⠀⠀⣀⣴⠦⣒⡤⡶⠒⠂⠂⠁⠁⠀⢰⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠸⠥⠴⠿⠷⣷⣟⠋⠁⠀⠀⢘⡌⠀⠀⠀⠀⠀⠀⠀⣾⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠘⡇⠀⠀⠀⠀⢸⠃⠀⠀⠀⠀⠀⠀⠀⠰⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
"#;

// Renders `HOT_GIRL` pinned to the bottom-right of `area`, cropped
// from the top/left when the area is too small to show it in full.
pub fn render_bottom_right(frame: &mut Frame, area: Rect) {
    let lines: Vec<&str> = HOT_GIRL.lines().filter(|l| !l.is_empty()).collect();
    let art_height: u16 = lines.len() as u16;
    let art_width: u16 = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;

    let avail_height: u16 = area.height;
    let avail_width: u16 = area.width;

    let visible_lines: Vec<&str> = if art_height > avail_height {
        let skip: usize = (art_height - avail_height) as usize;
        lines.into_iter().skip(skip).collect()
    } else {
        lines
    };

    let cropped_height: u16 = visible_lines.len() as u16;
    let render_width: u16 = art_width.min(avail_width);

    let target: Rect = bottom_right_rect(render_width, cropped_height, area);

    let text: Text<'_> = Text::from(
        visible_lines
            .into_iter()
            .map(Line::from)
            .collect::<Vec<_>>(),
    );

    let bg: Paragraph<'_> = Paragraph::new(text)
        .style(Style::default().fg(Color::Rgb(255, 133, 200)))
        .alignment(Alignment::Right);

    frame.render_widget(bg, target);
}

// Widest line of `FELIX_ARGYLE`, so callers can reserve columns for it
// before deciding how far up/left the art is allowed to climb.
pub fn felix_art_width() -> u16 {
    FELIX_ARGYLE
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.chars().count() as u16)
        .max()
        .unwrap_or(0)
}

// Renders `FELIX_ARGYLE` pinned to the bottom-right of `area`, cropped
// from the top/left when the area is too small to show it in full.
// Same crop/place logic as `render_bottom_right`, kept separate since
// each mascot gets its own tint.
pub fn render_felix_bottom_right(frame: &mut Frame, area: Rect) {
    let lines: Vec<&str> = FELIX_ARGYLE.lines().filter(|l| !l.is_empty()).collect();
    let art_height: u16 = lines.len() as u16;
    let art_width: u16 = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;

    let avail_height: u16 = area.height;
    let avail_width: u16 = area.width;

    let visible_lines: Vec<&str> = if art_height > avail_height {
        let skip: usize = (art_height - avail_height) as usize;
        lines.into_iter().skip(skip).collect()
    } else {
        lines
    };

    let cropped_height: u16 = visible_lines.len() as u16;
    let render_width: u16 = art_width.min(avail_width);

    let target: Rect = bottom_right_rect(render_width, cropped_height, area);

    let text: Text<'_> = Text::from(
        visible_lines
            .into_iter()
            .map(Line::from)
            .collect::<Vec<_>>(),
    );

    let bg: Paragraph<'_> = Paragraph::new(text)
        .style(Style::default().fg(Color::Rgb(200, 160, 255)))
        .alignment(Alignment::Right);

    frame.render_widget(bg, target);
}

fn bottom_right_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(width),
        ])
        .split(vertical[1])[1]
}

/// Renders `ASTOLFO_TOP` horizontally centered and pinned to the BOTTOM of
/// `area`, so the art meets the directions bar directly beneath it with no
/// gap. If the art is taller than `area`, rows are cropped off the TOP so the
/// half that joins the bar stays visible.
pub fn render_diva_top(frame: &mut Frame, area: Rect) {
    let lines: Vec<&str> = ASTOLFO_TOP.lines().filter(|l| !l.is_empty()).collect();
    let art_height: u16 = lines.len() as u16;
    let art_width: u16 = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;

    // How many rows we actually have to work with.
    let avail_height: u16 = area.height;
    let avail_width: u16 = area.width;

    // If the art is taller than available space, crop rows off the TOP
    // so the bottom half (what was being clipped) becomes visible.
    let visible_lines: Vec<&str> = if art_height > avail_height {
        let skip: usize = (art_height - avail_height) as usize;
        lines.into_iter().skip(skip).collect()
    } else {
        lines
    };

    let cropped_height: u16 = visible_lines.len() as u16;
    let render_width: u16 = art_width.min(avail_width);

    let target:Rect = bottom_centered_rect(render_width, cropped_height, area);

    let text: Text<'_> = Text::from(
        visible_lines
            .into_iter()
            .map(Line::from)
            .collect::<Vec<_ >>(),
    );

    let bg: Paragraph<'_> = Paragraph::new(text)
        .style(Style::default().fg(Color::Rgb(90, 90, 100)))
        .alignment(Alignment::Center);

    frame.render_widget(bg, target);
}

/// Renders `ASTOLFO_BOTTOM` horizontally centered and pinned to the TOP of
/// `area`, so it continues straight down from the directions bar above it.
/// If the art is taller than `area`, rows are cropped off the BOTTOM: the
/// skirt joining the bar stays, the feet lose out on short terminals.
pub fn render_diva_bottom(frame: &mut Frame, area: Rect) {
    let lines: Vec<&str> = ASTOLFO_BOTTOM.lines().filter(|l| !l.is_empty()).collect();
    let art_height: u16 = lines.len() as u16;
    let art_width: u16 = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;

    let avail_height: u16 = area.height;
    let avail_width: u16 = area.width;

    let visible_lines: Vec<&str> = if art_height > avail_height {
        lines.into_iter().take(avail_height as usize).collect()
    } else {
        lines
    };

    let cropped_height: u16 = visible_lines.len() as u16;
    let render_width: u16 = art_width.min(avail_width);

    let target: Rect = top_centered_rect(render_width, cropped_height, area);

    let text: Text<'_> = Text::from(
        visible_lines
            .into_iter()
            .map(Line::from)
            .collect::<Vec<_>>(),
    );

    let bg: Paragraph<'_> = Paragraph::new(text)
        .style(Style::default().fg(Color::Rgb(90, 90, 100)))
        .alignment(Alignment::Center);

    frame.render_widget(bg, target);
}

#[expect(unused)]
pub fn render_emo_pop_top(frame: &mut Frame, area: Rect) {
    let lines: Vec<&str> = EMO_POP_TOP.lines().filter(|l| !l.is_empty()).collect();
    let art_height: u16 = lines.len() as u16;
    let art_width: u16 = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;

    let avail_height: u16 = area.height;
    let avail_width: u16 = area.width;

    let visible_lines: Vec<&str> = if art_height > avail_height {
        let skip: usize = (art_height - avail_height) as usize;
        lines.into_iter().skip(skip).collect()
    } else {
        lines
    };

    let cropped_height: u16 = visible_lines.len() as u16;
    let render_width: u16 = art_width.min(avail_width);

    let target:Rect = centered_rect(render_width, cropped_height, area);

    let text: Text<'_> = Text::from(
        visible_lines
            .into_iter()
            .map(Line::from)
            .collect::<Vec<_ >>(),
    );

    let bg: Paragraph<'_> = Paragraph::new(text)
        .style(Style::default().fg(Color::Rgb(90, 90, 100)))
        .alignment(Alignment::Center);

    frame.render_widget(bg, target);
}

#[expect(unused)]
pub fn render_emo_pop_bottom(frame: &mut Frame, area: Rect) {
    let lines: Vec<&str> = EMO_POP_BOTTOM.lines().filter(|l| !l.is_empty()).collect();
    let art_height: u16 = lines.len() as u16;
    let art_width: u16 = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;

    let avail_height: u16 = area.height;
    let avail_width: u16 = area.width;

    let visible_lines: Vec<&str> = if art_height > avail_height {
        let skip: usize = (art_height - avail_height) as usize;
        lines.into_iter().skip(skip).collect()
    } else {
        lines
    };

    let cropped_height: u16 = visible_lines.len() as u16;
    let render_width: u16 = art_width.min(avail_width);

    let target: Rect = centered_rect(render_width, cropped_height, area);

    let text: Text<'_> = Text::from(
        visible_lines
            .into_iter()
            .map(Line::from)
            .collect::<Vec<_>>(),
    );

    let bg: Paragraph<'_> = Paragraph::new(text)
        .style(Style::default().fg(Color::Rgb(90, 90, 100)))
        .alignment(Alignment::Center);

    frame.render_widget(bg, target);
}

fn bottom_centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(width),
            Constraint::Fill(1),
        ])
        .split(vertical[1])[1]
}

fn top_centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(height),
            Constraint::Fill(1),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(width),
            Constraint::Fill(1),
        ])
        .split(vertical[0])[1]
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Fill(1),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(width),
            Constraint::Fill(1),
        ])
        .split(vertical[1])[1]
    }