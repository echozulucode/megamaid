@manage_plan @tree_first
Feature: Manage plan with tree-first view
  As a user finalizing cleanup
  I want a tree-first Manage Plan page
  So that I can clearly see what will be deleted, kept, or reviewed

  Background:
    Given a cleanup plan with entries for multiple subdirectories
    And protected paths are marked based on VCS markers and manifests

  Scenario: Navigate to Manage Plan (Tree tab)
    When the user opens the Manage Plan view
    Then the Tree tab is shown by default
    And the tree displays aggregate size and action counts per directory with color coding

  Scenario: Select a node and view details
    When the user selects a folder or file in the tree
    Then a detail pane shows its action, rule, reason, size, and modified time
    And the pane states whether the item will be deleted, kept, or reviewed

  Scenario: Apply actions with protection
    When the user tries to set delete on a protected path
    Then delete is disabled or rejected for that path
    And batch actions skip protected paths while applying to junk paths

  Scenario: Focus on pending deletes
    When the user enables the pending-deletes filter
    Then only items currently marked delete are shown in the tree and list
    And protected items remain visible as non-deletable

  Scenario: Jump from list to tree
    Given the user is on the List tab with filters applied
    When the user clicks an entry in the list
    Then the Tree tab focuses that same path and opens it in the hierarchy
