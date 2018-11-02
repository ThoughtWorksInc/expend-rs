#!/usr/bin/env bash
set -eu

exe=${1:?First argument must be the executable to test}

root="$(cd "${0%/*}" && pwd)"
exe="$root/../$exe"
# shellcheck disable=1090
source "$root/utilities.sh"
snapshot="$root/snapshots"

SUCCESSFULLY=0
WITH_FAILURE=1

(with "a valid 'from-file' sub-command"
  SCMD=(from-file file.yml job-type)
  (with "only a username set"
    it "fails with an error message as the password is required additionally" && {
      WITH_SNAPSHOT="$snapshot/failure-only-username-set" \
      expect_run ${WITH_FAILURE} "$exe" --user-id=user "${SCMD[@]}"
    }
  )
  (with "only a password set"
    it "fails with an error message as the username is required additionally" && {
      WITH_SNAPSHOT="$snapshot/failure-only-password-set" \
      expect_run ${WITH_FAILURE} "$exe" --user-secret=secret "${SCMD[@]}"
    }
  )
  (with "password and username provided with arguments"
    CREDS=(--user-id user --user-secret secret)
    (with "both dry-run and aLways-confirm set"
      it "fails with an error message indicating that those are mutually exclusive" && {
        WITH_SNAPSHOT="$snapshot/failure-dry-run-and-always-confirm-are-mutually-exclusive" \
        expect_run ${WITH_FAILURE} "$exe" -n -y "${CREDS[@]}" "${SCMD[@]}"
      }
    )
    (with "dry-run mode"
      DRY=-n
      (when "creating a post from a yml file"
        (with "a custom job type"
          it "produces the expected output and does nothing, and fails gracefully" && {
            WITH_SNAPSHOT="$snapshot/success-create-from-yml-file" \
            expect_run ${WITH_FAILURE} "$exe" $DRY "${CREDS[@]}" from-file <(echo 'somevalue: 42') job-type
          }
        )
        (with "no specifically set job type"
          it "produces the expected output and does nothing, and fails gracefully" && {
            WITH_SNAPSHOT="$snapshot/success-create-from-yml-file-default-jobtype" \
            expect_run ${WITH_FAILURE} "$exe" $DRY "${CREDS[@]}" from-file <(echo 'somevalue: 42')
          }
        )
      )
    )
    # TODO: -yes mode (which is prompted otherwise) with actual mock server being up
  )
)

(sandbox
  snapshot="$snapshot/context"
  context_dir=(--from ./contexts)
  (with "the 'context' sub-command"
    (with 'the list subcommand and no context available'
      it 'suggests that you should set the context first' && {
        WITH_SNAPSHOT="$snapshot/failure-list-without-any-context" \
        expect_run ${WITH_FAILURE} "$exe" contexts "${context_dir[@]}" list
      }
    )

    (when 'setting the default context (with a directory override for sandboxing)'
      it 'writes the expected file' && {
        WITH_SNAPSHOT="$snapshot/success-set-default" \
        expect_run ${SUCCESSFULLY} "$exe" contexts "${context_dir[@]}" set \
            --project 'the project name sans sub-project' \
            --email you@example.com
      }
      (when 'listing the available contexts'
        it 'shows the single context we just created' && {
          WITH_SNAPSHOT="$snapshot/success-list-contexts-default" \
          expect_run ${SUCCESSFULLY} "$exe" contexts "${context_dir[@]}" list
        }
      )
    )
  )
)
