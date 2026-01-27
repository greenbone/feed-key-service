Feature: Enterprise Feed Key

  Background:
    Given the service is running

  Rule: User is not authenticated

    Scenario: If the user is unauthenticated, it should not be possible to retrieve the feed key
      When I send a GET request to the key endpoint
      Then the response status code should be 401

    Scenario: If the user is unauthenticated, it should not be possible to delete the feed key
      When I send a DELETE request to the key endpoint
      Then the response status code should be 401

    Scenario: If the user is unauthenticated, it should not be possible to upload the feed key
      When I send a POST request to the key endpoint
      Then the response status code should be 401

    Scenario: If the user is unauthenticated, it should not be possible to update the feed key
      When I send a PUT request to the key endpoint
      Then the response status code should be 401

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

  Rule: Deleting the key

    Background:
      Given the user is authenticated

    Scenario: If the user is authenticated and a feed key exists, it should be possible to delete the feed key
      Given a feed key exists in the system
      When I send a DELETE request to the key endpoint
      Then the response status code should be 200
      And no feed key exists in the system

    Scenario: If the user is authenticated and no feed key exists, deleting the feed key should return successfully
      Given no feed key exists in the system
      When I send a DELETE request to the key endpoint
      Then the response status code should be 200
      And no feed key exists in the system

  Rule: Uploading and updating the key

    Background:
      Given the user is authenticated

    Scenario: If the user is authenticated, it should be possible to upload a new feed key
      Given no feed key exists in the system
      When I upload the feed key 'SOME-ENTERPRISE-FEED-KEY' via a PUT request to the key endpoint
      Then the response status code should be 200
      And a feed key exists in the system

    Scenario: If the user is authenticated, it should be possible to upload a new feed key via a form
      Given no feed key exists in the system
      When I upload the feed key 'SOME-ENTERPRISE-FEED-KEY' via a POST request to the key endpoint
      Then the response status code should be 200
      And a feed key exists in the system
