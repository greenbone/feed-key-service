Feature: Enterprise Feed Key

  Background:
    Given the service is running

  Rule: User is not authenticated

    Scenario: If the user is unauthenticated, it should not be possible to retrieve the feed key
      When I send a GET request to the key endpoint
      Then the response status code should be 401
      And the response body should be valid JSON
      And the JSON message should be "Unauthorized"

    Scenario: If the user is unauthenticated, it should not be possible to delete the feed key
      When I send a DELETE request to the key endpoint
      Then the response status code should be 401
      And the response body should be valid JSON
      And the JSON message should be "Unauthorized"

    Scenario: If the user is unauthenticated, it should not be possible to upload the feed key
      When I send a POST request to the key endpoint
      Then the response status code should be 401
      And the response body should be valid JSON
      And the JSON message should be "Unauthorized"

    Scenario: If the user is unauthenticated, it should not be possible to update the feed key
      When I send a PUT request to the key endpoint
      Then the response status code should be 401
      And the response body should be valid JSON
      And the JSON message should be "Unauthorized"

  Rule: Getting the key

    Background:
      Given the user is authenticated

    Scenario: If the user is authenticated, it should be possible to retrieve the feed key
      Given a feed key exists in the system
      When I send a GET request to the key endpoint
      Then the response status code should be 200
      And the response body should be 'SOME-ENTERPRISE-FEED-KEY'

    Scenario: If the user is authenticated and no feed key has been uploaded yet, the key retrieval should return an error
      Given no feed key exists in the system
      When I send a GET request to the key endpoint
      Then the response status code should be 404
      And the response body should be valid JSON
      And the JSON message should be "Key not available"

  Rule: Deleting the key

    Background:
      Given the user is authenticated

    Scenario: If the user is authenticated and a feed key exists, it should be possible to delete the feed key
      Given a feed key exists in the system
      When I send a DELETE request to the key endpoint
      Then the response status code should be 200
      And no feed key exists in the system
      And the response body should be valid JSON
      And the JSON message should be "Key deleted successfully"

    Scenario: If the user is authenticated and no feed key exists, deleting the feed key should return successfully
      Given no feed key exists in the system
      When I send a DELETE request to the key endpoint
      Then the response status code should be 200
      And no feed key exists in the system
      And the response body should be valid JSON
      And the JSON message should be "Key deleted successfully"

  Rule: Uploading the key

    Background:
      Given the user is authenticated

    Scenario: If the user is authenticated, it should be possible to upload a new feed key via a multipart form
      Given no feed key exists in the system
      When I post the field 'file' with content 'SOME-ENTERPRISE-FEED-KEY' to the key endpoint
      Then the response status code should be 200
      And a feed key exists in the system
      And the response body should be valid JSON
      And the JSON message should be "Key uploaded successfully"

    Scenario: If the user is authenticated and a feed key already exists, it should be possible to update the feed key via a form
      Given a feed key exists in the system
      When I post the field 'file' with content 'ANOTHER-ENTERPRISE-FEED-KEY' to the key endpoint
      Then the response status code should be 200
      And the feed key in the system should be 'ANOTHER-ENTERPRISE-FEED-KEY'
      And the response body should be valid JSON
      And the JSON message should be "Key uploaded successfully"

    Scenario: If the user is authenticated and not using a multipart form, it should be possible to upload a new feed key
      Given no feed key exists in the system
      When I upload the feed key 'SOME-ENTERPRISE-FEED-KEY' via a POST request to the key endpoint
      Then the response status code should be 400
      And no feed key exists in the system

    Scenario: If the user is authenticated and the key file is not writable, uploading a feed key should return an error
      Given a feed key exists in the system
      Given the feed key file is not writable
      When I post the field 'file' with content 'SOME-ENTERPRISE-FEED-KEY' to the key endpoint
      Then the response status code should be 500
      And the response body should be valid JSON
      And the JSON message should be "Internal server error: Key upload failed. File error."

    Scenario: If the user is authenticated, uploading a feed key with a wrong field should return an error
      Given a feed key exists in the system
      Given the feed key file is not writable
      When I post the field 'wrong_field' with content 'SOME-ENTERPRISE-FEED-KEY' to the key endpoint
      Then the response status code should be 400
      And the response body should be valid JSON
      And the JSON message should be "Bad request: Key upload failed. No file provided."

  Rule: Updating the key

    Background:
      Given the user is authenticated

    Scenario: If the user is authenticated, it should be possible to update the feed key
      Given no feed key exists in the system
      When I upload the feed key 'SOME-ENTERPRISE-FEED-KEY' via a PUT request to the key endpoint
      Then the response status code should be 200
      And a feed key exists in the system
      And the response body should be valid JSON
      And the JSON message should be "Key uploaded successfully"

    Scenario: If the user is authenticated and a feed key already exists, it should be possible to update the feed key
      Given a feed key exists in the system
      When I upload the feed key 'ANOTHER-ENTERPRISE-FEED-KEY' via a PUT request to the key endpoint
      Then the response status code should be 200
      And the feed key in the system should be 'ANOTHER-ENTERPRISE-FEED-KEY'
      And the response body should be valid JSON
      And the JSON message should be "Key uploaded successfully"

    Scenario: If the user is authenticated and the key file is not writable, updating the feed key should return an error
      Given a feed key exists in the system
      Given the feed key file is not writable
      When I upload the feed key 'SOME-ENTERPRISE-FEED-KEY' via a PUT request to the key endpoint
      Then the response status code should be 500
      And the response body should be valid JSON
      And the JSON message should be "Internal server error: Key upload failed. File error."
