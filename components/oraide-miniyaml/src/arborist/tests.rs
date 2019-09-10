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

use unindent::unindent;

use language_reporting::Severity;

use mltt_span::{
    Files,
};

use crate::{
    Lexer,
    Parser,
    Tree,
    arborist::{
        Arborist,
    },
};

#[test]
fn general_spaces_only_indent_test() {
    let _ = env_logger::try_init(); // ignore failure

    // Arrange
    let src = unindent("
        A2:
            B3:

        C5:
            D6:
                E7:
            F8:
                G9:
            H10:

        I12:
    ");

    const EXPECTED_ARENA_COUNT: usize =
        1 + // sentinel node
       11;  // number of lines in `src`

    let mut files = Files::new();
    let file_id = files.add("test", src);
    let file = &files[file_id];

    let lexer = Lexer::new(file);
    let tokens = lexer.collect::<Vec<_>>();
    let parser = Parser::new(tokens.into_iter());
    let nodes = parser.collect::<Vec<_>>();

    let mut arborist = Arborist::new(nodes.into_iter());

    // Act
    let Tree { node_ids, arena } = arborist.build_tree();

    // Assert
    let actual_arena_count = arena.count();
    assert_eq!(actual_arena_count, EXPECTED_ARENA_COUNT, "{:#?}", arena.iter().map(|arena_node| &arena_node.data).collect::<Vec<_>>());
    assert_eq!(node_ids.len(), actual_arena_count);

    // We actually added each node ID to the arena
    for id in node_ids {
        assert!(arena.get(id).is_some(), "{}", id);
    }

    let diags = arborist.take_diagnostics();
    let err_and_bug_diags = diags.iter()
        .filter(|diag| diag.severity == Severity::Error || diag.severity == Severity::Bug)
        .collect::<Vec<_>>();

    assert!(
        err_and_bug_diags.is_empty(),
        "There should be no `error` or `bug` diagnostics: {:#?}",
        diags
    );
}

#[test]
fn general_tabs_only_indent_test() {
    let _ = env_logger::try_init(); // ignore failure

    // Arrange
    let src = unindent("
        A2:
        \tB3:

        C5:
        \tD6:
        \t\tE7:
        \tF8:
        \t\tG9:
        \tH10:

        I12:
    ");

    const EXPECTED_ARENA_COUNT: usize =
        1 + // sentinel node
       11;  // number of lines in `src`

    let mut files = Files::new();
    let file_id = files.add("test", src);
    let file = &files[file_id];

    let lexer = Lexer::new(file);
    let tokens = lexer.collect::<Vec<_>>();
    let parser = Parser::new(tokens.into_iter());
    let nodes = parser.collect::<Vec<_>>();

    let mut arborist = Arborist::new(nodes.into_iter());

    // Act
    let Tree { node_ids, arena } = arborist.build_tree();

    // Assert
    let actual_arena_count = arena.count();
    assert_eq!(actual_arena_count, EXPECTED_ARENA_COUNT, "{:#?}", arena.iter().map(|arena_node| &arena_node.data).collect::<Vec<_>>());
    assert_eq!(node_ids.len(), actual_arena_count);

    // We actually added each node ID to the arena
    for id in node_ids {
        assert!(arena.get(id).is_some(), "{}", id);
    }

    let diags = arborist.take_diagnostics();
    let err_and_bug_diags = diags.iter()
        .filter(|diag| diag.severity == Severity::Error || diag.severity == Severity::Bug)
        .collect::<Vec<_>>();

    assert!(
        err_and_bug_diags.is_empty(),
        "There should be no `error` or `bug` diagnostics: {:#?}",
        diags
    );
}