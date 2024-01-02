project('outbox', 'rust', meson_version: '>= 0.59.0')

subdir('man')

build_root = meson.project_build_root()
source_root = meson.project_source_root()

cargo = find_program('cargo', required: true)
cargo_sources = files(
  'Cargo.toml',
  'Cargo.lock',
)
cargo_options = [ '--manifest-path', source_root / 'Cargo.toml' ]
cargo_options += [ '--target-dir', build_root / 'src' ]

if get_option('buildtype') == 'debug'
  rust_target = 'debug'
else
  cargo_options += [ '--release' ]
  rust_target = 'release'
endif

cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: 'src',
  console: true,
  command: [
    cargo, 'build',
    cargo_options,
  ]
)

copy_binary = custom_target(
  'cp-binary',
  depends: cargo_build,
  build_by_default: true,
  build_always_stale: true,
  install: true,
  install_dir: get_option('bindir'),
  output: 'outboxd',
  command: [
    'cp',
    'src' / rust_target / 'outboxd',
    '@OUTPUT@',
  ],
)
