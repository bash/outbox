gzip = find_program('gzip', required: false)
man_pages = [
    ['outboxd', '1'],
    ['garden.tau.Outbox', '1']
]

if not gzip.found()
    warning('gzip not found; man pages will not be generated')
else
    foreach tuple : man_pages
        stem = tuple[0]
        section = tuple[1]
        man = stem + '.' + section
        custom_target(
            man,
            build_by_default: true,
            input: man,
            output: man + '.gz',
            install: true,
            install_dir: get_option('mandir') / ('man' + section),
            command: [meson.current_source_dir() / 'compress.sh', gzip, '@INPUT@', '@OUTPUT@']
        )
    endforeach
endif
