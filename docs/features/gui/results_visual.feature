@phase4_3 @results
Feature: Visualize scan results as a tree
  As a user reviewing space usage
  I want to see candidates organized in a tree with summaries
  So that I can quickly focus on large or risky areas

  Background:
    Given a cleanup plan with entries for multiple subdirectories

  Scenario: Expand directories to inspect nested entries
    Given the tree view shows aggregate size and action counts per directory
    When the user expands a directory
    Then its child directories and entries are shown in the tree
    And protected entries are labeled as protected

  Scenario: Filter and search results
    Given the results list includes delete, review, and keep entries
    When the user applies an action filter and searches by path text
    Then only entries matching the filter and search text are shown
