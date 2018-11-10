#!/usr/bin/env bash
set -eu

exe=${1:?First argument must be the executable to test}

root="$(cd "${0%/*}" && pwd)"
exe="$root/../$exe"
# shellcheck disable=1090
source "$root/utilities.sh"
snapshot="$root/snapshots"
fixture="$root/fixtures"

SUCCESSFULLY=0
WITH_FAILURE=1


(with "a 'post' subcommand"
  CREDS=(--user-id user --user-secret secret)
  DRY=-n
  (with "the 'from-file' subcommand"
    SCMD=(from-file file.yml job-type)
    (with "only a username set"
      it "fails with an error message as the password is required additionally" && {
        WITH_SNAPSHOT="$snapshot/failure-only-username-set" \
        expect_run ${WITH_FAILURE} "$exe" post --user-id=user "${SCMD[@]}"
      }
    )
    (with "only a password set"
      it "fails with an error message as the username is required additionally" && {
        WITH_SNAPSHOT="$snapshot/failure-only-password-set" \
        expect_run ${WITH_FAILURE} "$exe" post --user-secret=secret "${SCMD[@]}"
      }
    )
    (with "password and username provided with arguments"
      (with "both dry-run and aLways-confirm set"
        it "fails with an error message indicating that those are mutually exclusive" && {
          WITH_SNAPSHOT="$snapshot/failure-dry-run-and-always-confirm-are-mutually-exclusive" \
          expect_run ${WITH_FAILURE} "$exe" post -n -y "${CREDS[@]}" "${SCMD[@]}"
        }
      )
      (with "dry-run mode"
        (when "creating a post from a yml file"
          (with "an unset context"
            (with "a custom job type"
              it "produces the expected output and does nothing, and fails gracefully" && {
                WITH_SNAPSHOT="$snapshot/success-create-from-yml-file" \
                expect_run ${WITH_FAILURE} "$exe" post $DRY "${CREDS[@]}" from-file <(echo 'somevalue: 42') job-type
              }
            )
            (with "no specifically set job type"
              it "produces the expected output and does nothing, and fails gracefully" && {
                WITH_SNAPSHOT="$snapshot/success-create-from-yml-file-default-jobtype" \
                expect_run ${WITH_FAILURE} "$exe" post $DRY "${CREDS[@]}" from-file <(echo 'somevalue: 42')
              }
            )
          )
          (sandbox
            (with "the default context set"
              step "(setting the context)"
              expect_run ${SUCCESSFULLY} "$exe" context --at . set --email me@example.com --project 'project code'

              (when "creating a post from a yml file with explicit context"
                it "produces the expected output with the context integrated into the payload, does nothing, and fails gracefully" && {
                  WITH_SNAPSHOT="$snapshot/success-create-from-yml-file-default-jobtype-with-context" \
                  expect_run ${WITH_FAILURE} "$exe" post --context-dir . $DRY "${CREDS[@]}" from-file --context  default "$fixture/transaction-list.json"
                }
              )
            )
          )
        )
      )
      # TODO: -yes mode (which is prompted otherwise) with actual mock server being up
    )
  )

  (with "the 'per-diem' subcommand"
    (with "dry-run mode"
      (sandbox 
        (with "no context available"
          it "fails telling how to create a context" && {
            WITH_SNAPSHOT="$snapshot/failure-create-per-diem-missing-context" \
            expect_run ${WITH_FAILURE} "$exe" post --context-dir . $DRY "${CREDS[@]}" per-diem weekdays fullday
          }
        )
        (with "a default context available (and the time set to a known date)"
          WEEKDATE=(--weekdate 1972-09-02)
          step "(setting the context)"
          expect_run ${SUCCESSFULLY} "$exe" context --at . set --email me@example.com --project 'project code'

          (when "using the 'weekdays' period"
            (when "using the 'fullday' kind"
              it "succeeds and creates a properly formatted payload" && {
                WITH_SNAPSHOT="$snapshot/success-create-per-diem-weekdays-fullday" \
                expect_run ${WITH_FAILURE} "$exe" post --context-dir . $DRY "${CREDS[@]}" "${WEEKDATE[@]}" per-diem weekdays fullday
              }
            )
          )
          (when "using the flexible 'range from-to' period"
            (when "using the 'fullday' kind"
              (with "no comment"
                it "succeeds and creates a properly formatted payload" && {
                  WITH_SNAPSHOT="$snapshot/success-create-per-diem-range-fullday" \
                  expect_run ${WITH_FAILURE} "$exe" post --context-dir . $DRY "${CREDS[@]}" "${WEEKDATE[@]}" per-diem mon-sun fullday
                }
              )
              (with "a comment"
                it "succeeds and creates a properly formatted payload" && {
                  WITH_SNAPSHOT="$snapshot/success-create-per-diem-range-fullday-with-comment" \
                  expect_run ${WITH_FAILURE} "$exe" post --context-dir . $DRY "${CREDS[@]}" "${WEEKDATE[@]}" per-diem mon-sun fullday --comment "custom comment"
                }
              )
            )
          )
          (when "using the 'any-given-days' period"
            (when "using the 'fullday' kind"
              (with "no comment"
                it "succeeds and creates a properly formatted payload" && {
                  WITH_SNAPSHOT="$snapshot/success-create-per-diem-given-days-fullday" \
                  expect_run ${WITH_FAILURE} "$exe" post --context-dir . $DRY "${CREDS[@]}" "${WEEKDATE[@]}" per-diem mon,wednesday,fri fullday
                }
              )
              (with "a comment"
                it "succeeds and creates a properly formatted payload" && {
                  WITH_SNAPSHOT="$snapshot/success-create-per-diem-given-days-fullday-with-comment" \
                  expect_run ${WITH_FAILURE} "$exe" post --context-dir . $DRY "${CREDS[@]}" "${WEEKDATE[@]}" per-diem mon,wednesday,fri fullday -m "custom comment"
                }
              )
            )
          )
          (when "using the 'single-day' period"
            for kind in fullday breakfast arrival departure daytrip lunch dinner; do
              (when "using the '$kind' kind and when subtracting it"
                it "succeeds and creates a properly formatted payload" && {
                  WITH_SNAPSHOT="$snapshot/success-create-per-diem-single-day-$kind" \
                  expect_run ${WITH_FAILURE} "$exe" post --context-dir . $DRY "${CREDS[@]}" "${WEEKDATE[@]}" per-diem --subtract thursday $kind
                }
              )
            done

            (with "a custom comment"
              it "succeeds and uses the comment exclusively" && {
                WITH_SNAPSHOT="$snapshot/success-create-per-diem-single-day-lunch-with-comment" \
                expect_run ${WITH_FAILURE} "$exe" post --context-dir . $DRY "${CREDS[@]}" "${WEEKDATE[@]}" per-diem --subtract thursday lunch --comment 'lunch date'
              }
            )
          )

          (when "using the an unknown per-diem period"
            it "fails gracefully" && {
              WITH_SNAPSHOT="$snapshot/failure-create-per-diem-unknown-period" \
              expect_run ${WITH_FAILURE} "$exe" post --context-dir . $DRY "${CREDS[@]}" per-diem foobar fullday
            }
          )

          (when "using the an unknown per-diem kind"
            it "fails gracefully" && {
              WITH_SNAPSHOT="$snapshot/failure-create-per-diem-unknown-kind" \
              expect_run ${WITH_FAILURE} "$exe" post --context-dir . $DRY "${CREDS[@]}" per-diem weekdays foobar
            }
          )
        )
      )
    )
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
      it 'succeeds' && {
        WITH_SNAPSHOT="$snapshot/success-set-default" \
        expect_run ${SUCCESSFULLY} "$exe" contexts "${context_dir[@]}" set \
            --project 'the project name sans sub-project' \
            --email you@example.com
      }
      it 'writes the expected file' && {
        expect_snapshot "$snapshot/context-dir-with-default" .
      }

      (when 'listing the available contexts'
        it 'shows the single context we just created' && {
          WITH_SNAPSHOT="$snapshot/success-list-contexts-default" \
          expect_run ${SUCCESSFULLY} "$exe" contexts "${context_dir[@]}" list
        }
      )
      (when 'getting the newly created context'
        it 'shows it as yaml' && {
          WITH_SNAPSHOT="$snapshot/success-get-context-default" \
          expect_run ${SUCCESSFULLY} "$exe" contexts "${context_dir[@]}" get
        }
      )
      (when 'setting another named context with all optional flags set'
        it 'succeeds' && {
          expect_run ${SUCCESSFULLY} "$exe" contexts "${context_dir[@]}" set \
              --name other-client \
              --project 'some other project name' \
              --email me@example.com \
              --country Germany \
              --travel-tag-name Travel \
              --travel-tag-unbillable \
              --category-per-diems-name "perdiem category name"
        }
        it 'writes the expected file' && {
          expect_snapshot "$snapshot/context-dir-multiple-contexts" .
        }
        it 'lists all now available contexts' && {
          WITH_SNAPSHOT="$snapshot/success-list-contexts-multiple" \
          expect_run ${SUCCESSFULLY} "$exe" contexts "${context_dir[@]}" list
        }
      )
    )
  )
)
