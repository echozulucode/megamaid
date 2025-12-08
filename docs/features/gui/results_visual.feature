@phase4_3 @results
Feature: Summarize scan results
  As a user reviewing space usage
  I want a concise summary of scan and plan outcomes
  So that I can decide whether to drill into Manage Plan

  Background:
    Given a cleanup plan exists for the last scan

  Scenario: View summary cards and base path
    When the user opens Results
    Then the page shows files scanned, candidate count, potential space saved, and base path
    And counts for delete, review, and keep are displayed

  Scenario: Filter and search the list
    Given the results list includes delete, review, and keep entries
    When the user applies an action filter and searches by path text
    Then only entries matching the filter and search text are shown
