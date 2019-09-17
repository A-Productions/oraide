// This file is part of oraide.  See <https://github.com/Phrohdoh/oraide>.
// 
// oraide is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License version 3
// as published by the Free Software Foundation.
// 
// oraide is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License
// along with oraide.  If not, see <https://www.gnu.org/licenses/>.

use std::{
    env,
    io::{
        Read,
    },
    fs::{
        File,
    },
    path::{
        Path,
        PathBuf,
    },
};

use oraide_span::{
    FileId,
};

use oraide_parser_miniyaml::{
    TextFilesCtxExt as _,
};

use oraide_query_system::OraideDatabase;

mod commands;
use commands::{
    Parse,
    FindDefinition,
    Hover,
    ListGames,
};

mod ide;

// NOTE: This looks an awful lot like a binary package, but is indeed a library.
//       This will be invoked by a top-level bin target (the overarching `oraide` crate),
//       see `<root>/src/main.rs` for more context.
/// Run the command given by the user!
pub fn main() {
    let mut args = env::args().skip(1);

    let cmd = match args.next() {
        Some(cmd) => cmd,
        _ => {
            print_usage_instructions();
            return;
        },
    };

    match cmd.as_ref() {
        "parse" => {
            let file_paths = args.into_iter()
                .map(PathBuf::from)
                .collect::<Vec<_>>();

            let file_count = file_paths.len();

            let parse = Parse::new(file_paths)
                .expect("Failed to setup parse");

            let start = std::time::Instant::now();
            parse.run();
            println!("[info] took {:?} to parse {} file(s)", start.elapsed(), file_count);
        },
        "find-def" | "find-defs" | "find-definition" | "find-definitions" => {
            let project_root_dir = match args.next() {
                Some(n) => PathBuf::from(n),
                _ => {
                    eprintln!("Please provide a path to a project root directory");
                    return;
                },
            };

            let name_to_find = match args.next() {
                Some(n) => n,
                _ => {
                    eprintln!("Please provide an item name to find (ex: E1)");
                    return;
                },
            };

            let find_def = FindDefinition::new(name_to_find.clone(), project_root_dir)
                .expect("Failed to setup find-definition");

            let start = std::time::Instant::now();
            find_def.run();
            println!("[info] took {:?} to look for definition(s) of `{}`", start.elapsed(), name_to_find);
        },
        "hover" => {
            let root_dir = match args.next() {
                Some(path) => path.into(),
                _ => {
                    eprintln!();
                    eprintln!("Please provide the <root-dir> parameter (see below for more information)");
                    eprintln!();
                    print_usage_instructions();
                    return;
                },
            };

            let rel_file_path = match args.next() {
                Some(path) => path.into(),
                _ => {
                    eprintln!();
                    eprintln!("Please provide the <rel-file-path> parameter (see below for more information)");
                    eprintln!();
                    print_usage_instructions();
                    return;
                },
            };

            let line_idx = match args.next().map(|num_str| num_str.parse()) {
                Some(Ok(num)) => num,
                _ => {
                    eprintln!();
                    eprintln!("Please provide the <line-number> parameter as a non-negative integer (see usage for more information)");
                    eprintln!();
                    print_usage_instructions();
                    return;
                },
            };

            let col_idx = match args.next().map(|num_str| num_str.parse()) {
                Some(Ok(num)) => num,
                _ => {
                    eprintln!();
                    eprintln!("Please provide the <column-number> parameter as a non-negative integer (see usage for more information)");
                    eprintln!();
                    print_usage_instructions();
                    return;
                },
            };

            let hover = Hover::new(root_dir, rel_file_path, line_idx, col_idx)
                .expect("Failed to setup hover");

            hover.run();
        },
        "ide" => {
            ide::ide();
        },
        "lint" => {
            match args.next() {
                Some(_file_path) => {
                    unimplemented!("linting")
                },
                _ => eprintln!("Please provide a file path to lint"),
            }
        },
        "list-games" => {
            let root_dir: PathBuf = match args.next() {
                Some(path) => path.into(),
                _ => {
                    eprintln!();
                    eprintln!("Please provide the <root-dir> parameter (see below for more information)");
                    eprintln!();
                    print_usage_instructions();
                    return;
                },
            };

            let list_games = ListGames::new_with_root_dir(root_dir);
            list_games.run();
        },
        other => {
            eprintln!("!!! got `{}`", other);
            print_usage_instructions();
        }
    }
}

fn print_usage_instructions() {
    eprintln!("Usage:");
    eprintln!("  ora ide                                                                 - run the OpenRA language server / IDE support");
    eprintln!("  ora parse      <file-path>                                              - print all definitions (top-level items) in a file");
    eprintln!("  ora find-defs  <project-root-path> <item-name>                          - find all definitions with name <item-name> in <project-root-path>");
    eprintln!("  ora hover      <root-dir> <rel-file-path> <line-number> <column-number> - print hover data for the token at <root-dir>/<rel-file-path>:<line-number>:<column-number>");
    eprintln!("    example: ora hover /path/to/project/root/dir rules/infantry.yaml 15 8");
    eprintln!("  ora list-games <root-dir>                                               - print IDs and names for all games in <root-dir>/mods/");
  //eprintln!("  ora lint <file-path>                                                   - unimplemented");
}

/// Read the contents of `file_path` and add it to `db`, creating and returning
/// the newly-created [`FileId`], returning `Err(String)` if something goes wrong.
///
/// [`FileId`]: ../oraide_span/struct.FileId.html
pub(crate) fn add_file(db: &mut OraideDatabase, file_path: &Path) -> Result<FileId, String> {
    let text = {
        let mut file = File::open(file_path)
            .map_err(|e| format!("Error opening `{}`: {}", file_path.display(), e))?;

        let mut text = String::new();
        file.read_to_string(&mut text)
            .map_err(|e| format!("Error reading `{}`: {}", file_path.display(), e))?;

        text
    };

    let file_id = db.add_text_file(file_path.to_string_lossy(), text);

    Ok(file_id)
}