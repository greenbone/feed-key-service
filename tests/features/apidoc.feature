Feature: Provide API Documentation

  Background:
    Given the service is running

  Scenario: If the user requests the API documentation endpoint, it should return the OpenAPI specification
    Given the API documentation is enabled
    When I send a GET request to the API documentation
    Then the response status code should be 200
    And the response body should be valid JSON
    And the response body should contain the OpenAPI version "3.1.0"

  Scenario: If the user requests the API documentation endpoint when the feature is disabled, it should return a 404 error
    When I send a GET request to the API documentation
    Then the response status code should be 404

  Scenario: If the user requests the swagger UI, it should return the Swagger UI HTML page
    Given the API documentation is enabled
    When I send a GET request to the swagger UI
    Then the response status code should be 200

  Scenario: If the user requests the swagger UI when the feature is disabled, it should return a 404 error
    When I send a GET request to the swagger UI
    Then the response status code should be 404
