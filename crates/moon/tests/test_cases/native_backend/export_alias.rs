// moon: The build system and package manager for MoonBit.
// Copyright (C) 2024 International Digital Economy Academy
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
// For inquiries, you can contact us via e-mail at jichuruanjian@idea.edu.cn.

use crate::{TestDir, get_stdout};

/// Test that native backend export aliases generate the postprocess command.
///
/// When `moon.pkg.json` has `link.native.exports` with alias entries like
/// "hello:my_hello", the build command for `moonc link-core` should be chained
/// with `moon tool postprocess-native-exports` to generate C wrapper functions.
#[test]
fn test_exports_in_native_backend() {
    let dir = TestDir::new("native_backend/export_alias");

    let out = get_stdout(
        &dir,
        [
            "build",
            "--dry-run",
            "--nostd",
            "--target",
            "native",
            "--sort-input",
        ],
    );

    // The link-core command should be chained with postprocess-native-exports
    assert!(
        out.contains("postprocess-native-exports"),
        "Expected postprocess-native-exports in dry-run output, got:\n{}",
        out
    );
    // Verify the exports are passed correctly
    assert!(
        out.contains("hello:my_hello"),
        "Expected hello:my_hello in dry-run output, got:\n{}",
        out
    );
    assert!(
        out.contains("do_nothing:my_do_nothing"),
        "Expected do_nothing:my_do_nothing in dry-run output, got:\n{}",
        out
    );
}
