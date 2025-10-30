set export

RUSTFLAGS := "-Dwarnings"
RUSTDOCFLAGS := "--cfg docs -Dwarnings"

pre-commit *FLAGS:
    ./scripts/pre-commit.sh {{FLAGS}}

doc *FLAGS:
    cargo +nightly doc -Zrustdoc-map --all-features {{FLAGS}}
