@phase4_2 @scan
Feature: Scan workspace for cleanup candidates
  As a desktop user
  I want to start and resume scans of a chosen workspace
  So that I can review size and candidate counts before cleaning

  Background:
    Given the user is running the desktop app with the Tauri runtime available

  Scenario: Start a scan after choosing a directory
    Given the user selects a workspace directory
    When the user starts a scan
    Then the system reports files scanned and total size
    And the system generates a cleanup plan with delete, review, and keep counts

  Scenario: Resume with the last scanned directory on launch
    Given a prior scan was completed for a directory
    When the app is opened again
    Then the last scanned directory is preselected
    And the user can start a new scan without re-picking the path
