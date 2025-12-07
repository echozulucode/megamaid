@phase4_4 @persistence
Feature: Save and load cleanup plans
  As a user collaborating on cleanup
  I want to save and reopen plans
  So that edits are reusable across sessions

  Scenario: Save plan to disk
    Given a cleanup plan has entries to review
    When the user chooses to save and confirms a path
    Then the plan is written to the chosen file
    And the app shows the saved file path

  Scenario: Cancel save without writing
    Given a cleanup plan has entries to review
    When the user cancels the save dialog
    Then no file is written
    And the app reports that saving was canceled

  Scenario: Load an existing plan
    Given an existing plan file on disk
    When the user loads the plan
    Then the entries and summary counts appear in the app
