# ui-gpui (skeleton, currently blocked upstream)

Goal: native GPUI frontend for kutascribe (Zed's GPU UI).

Status: `gpui = "*"` on crates.io is 0.2.2 (2024), pulls `xattr 0.2.3` which fails on Rust 1.98:
```
error[E0425]: cannot find value ENOATTR in crate libc
```

Real GPUI lives in https://github.com/zed-industries/zed/tree/main/crates/gpui, not crates.io.
To use it you must vendor Zed:

```bash
# heavy (~2GB checkout, 10+ min build) — do once
git submodule add https://github.com/zed-industries/zed third_party/zed
# in ui-gpui/Cargo.toml:
# gpui = { path = "../../third_party/zed/crates/gpui" }
```

`src/main.rs` here is the HelloWorld from https://gpui.rs adapted to kutascribe
(status, timestamps toggle, chunk slider, device picker). It will compile once
pointed at a Zed checkout.

Recommendation: ship `../app` (eframe, builds today, 247M debug / ~30M release)
for Linux/Windows/Mac now, migrate to GPUI after core stabilises. Both call the
same `kutascribe` single binary, so no logic fork.
