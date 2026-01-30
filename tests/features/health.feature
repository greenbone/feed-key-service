Feature: Service is Healthy

  Scenario: If the service is healthy, it should return a success response
    Given the service is running
    When I send a GET request to the health endpoint
    Then the response status code should be 200
    And the response body should be valid JSON
    And the response body should be '{"status":"success","message":"OK server is healthy"}'
