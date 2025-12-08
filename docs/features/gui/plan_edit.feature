@phase4_4 @plan
Feature: Review and edit cleanup actions in Manage Plan
  As a user finalizing a cleanup plan
  I want to adjust actions safely
  So that only junk is deleted and important paths are kept

  Background:
    Given a generated cleanup plan is loaded
    And protected paths are identified based on VCS markers and manifests

  Scenario: Inline action changes are applied in the tree or list
    When the user changes an entry to keep, review, or delete from the Tree or List tab
    Then the plan updates that single entry action
    And the plan summary counts reflect the change

  Scenario: Delete is blocked for protected paths
    When the user attempts to set delete on a protected path
    Then the delete action is disabled or rejected
    And the plan retains a non-delete action for that path

  Scenario: Apply batch actions
    When the user applies the batch delete for build artifacts
    Then entries flagged as build artifacts switch to delete unless protected
    And protected paths remain keep
