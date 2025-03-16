// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// @title Contract Ledger
/// @notice Records and verifies contract events on the blockchain
contract ContractLedger {
    /// @notice Structure for contract events
    struct Event {
        string eventType;
        string data;
        uint256 timestamp;
    }

    /// @notice Mapping from contract ID to its events
    mapping(string => Event[]) private contractEvents;
    
    /// @notice Mapping from contract ID to its current state hash
    mapping(string => string) private contractStates;

    /// @notice Emitted when a new event is recorded
    event EventRecorded(
        string indexed contractId,
        string eventType,
        string data,
        uint256 timestamp
    );

    /// @notice Records a new contract event
    /// @param eventType Type of the event (CREATED, SIGNED, UPDATED, VOIDED)
    /// @param contractId Unique identifier of the contract
    /// @param data JSON encoded event data
    function recordEvent(
        string memory eventType,
        string memory contractId,
        string memory data
    ) public returns (bool) {
        require(bytes(contractId).length > 0, "Contract ID cannot be empty");
        require(bytes(eventType).length > 0, "Event type cannot be empty");

        Event memory newEvent = Event({
            eventType: eventType,
            data: data,
            timestamp: block.timestamp
        });

        contractEvents[contractId].push(newEvent);

        // Update contract state if this is a creation or update event
        if (keccak256(bytes(eventType)) == keccak256(bytes("CREATED")) ||
            keccak256(bytes(eventType)) == keccak256(bytes("UPDATED"))) {
            // Extract content hash from data
            // Note: This assumes the data is a JSON object containing a content_hash field
            contractStates[contractId] = data;
        }

        emit EventRecorded(contractId, eventType, data, block.timestamp);
        return true;
    }

    /// @notice Retrieves all events for a contract
    /// @param contractId Unique identifier of the contract
    /// @return Array of events (type, data, timestamp)
    function getContractEvents(string memory contractId)
        public
        view
        returns (Event[] memory)
    {
        require(bytes(contractId).length > 0, "Contract ID cannot be empty");
        return contractEvents[contractId];
    }

    /// @notice Verifies if a contract's current state matches the expected hash
    /// @param contractId Unique identifier of the contract
    /// @param expectedHash Expected hash of the contract state
    /// @return bool indicating if the state matches
    function verifyContractState(
        string memory contractId,
        string memory expectedHash
    ) public view returns (bool) {
        require(bytes(contractId).length > 0, "Contract ID cannot be empty");
        require(bytes(expectedHash).length > 0, "Expected hash cannot be empty");

        string memory currentState = contractStates[contractId];
        return keccak256(bytes(currentState)) == keccak256(bytes(expectedHash));
    }

    /// @notice Gets the number of events for a contract
    /// @param contractId Unique identifier of the contract
    /// @return Number of events
    function getEventCount(string memory contractId)
        public
        view
        returns (uint256)
    {
        return contractEvents[contractId].length;
    }

    /// @notice Gets a specific event for a contract
    /// @param contractId Unique identifier of the contract
    /// @param eventIndex Index of the event to retrieve
    /// @return Event type, data, and timestamp
    function getEvent(string memory contractId, uint256 eventIndex)
        public
        view
        returns (string memory, string memory, uint256)
    {
        require(eventIndex < contractEvents[contractId].length, "Event index out of bounds");
        Event memory event = contractEvents[contractId][eventIndex];
        return (event.eventType, event.data, event.timestamp);
    }
} 