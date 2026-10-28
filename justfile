format:
    just --fmt --unstable
    cargo fmt

[arg("fix", long="fix", value="true")]
lint fix="false":
    #!/usr/bin/env bash
    set -euxo pipefail

    disallow=(
        warnings
        deprecated
        clippy::all
        clippy::correctness
        clippy::suspicious
        clippy::complexity
        clippy::perf
        clippy::style
        clippy::absolute_paths
        clippy::pedantic
        clippy::nursery
    )
    allow=(
        clippy::too_many_arguments
    )

    if [ "{{ fix }}" = "true" ]; then
        cargo clippy "${scope[@]}" --fix --allow-dirty \
            -- ${disallow[@]/#/-D } ${allow[@]/#/-A }
    else
        cargo clippy "${scope[@]}" \
            -- ${disallow[@]/#/-D } ${allow[@]/#/-A }
    fi
