@phase4_safety @protected
Feature: Protect source and project roots
  As a user preserving projects
  I want megamaid to avoid deleting source and repo roots
  So that only disposable junk is removed

  Background:
    Given the workspace contains both project roots and junk directories like node_modules and target

  Scenario: Skip deleting the workspace root
    When a scan generates a cleanup plan
    Then the plan does not include the base path or "."
    And the plan never proposes deleting the workspace root

  Scenario: Protect repositories and manifests
    When a path contains VCS markers or manifest files
    Then the default action is review or keep, not delete
    And delete buttons for those paths are disabled in the plan UI

  Scenario: Allow deleting common build artifacts
    When a path matches a known junk directory such as node_modules or target
    Then the plan may set delete by default
    And batch delete for build artifacts applies only to those junk paths
