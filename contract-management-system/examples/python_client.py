"""
Contract Management System Python Client

This module provides a Python client for interacting with the Contract Management System API.
It handles DID-based authentication, contract operations, and blockchain event verification.

Features:
    - Async API using httpx
    - DID registration and authentication
    - Contract creation and management
    - Event verification
    - JWT token handling

Example:
    ```python
    async def main():
        client = ContractClient("https://api.example.com")
        
        # Register and authenticate
        await client.register_did("did:example:123", "public_key")
        await client.authenticate("did:example:123", "private_key")
        
        # Create contract
        contract_data = {
            "title": "Test Contract",
            "description": "Example contract",
            "consumer_did": "did:example:456",
            "terms": "Contract terms...",
            "valid_from": "2024-03-21T00:00:00Z",
            "valid_until": "2025-03-21T00:00:00Z"
        }
        result = await client.create_contract(contract_data)
    ```
"""

from dataclasses import dataclass
import httpx
from typing import Optional, Dict, Any
import jwt
from eth_account import Account
from eth_account.messages import encode_defunct
from datetime import datetime, timezone

@dataclass
class ContractClient:
    """
    Client for interacting with the Contract Management System API.
    
    Attributes:
        base_url (str): Base URL of the API endpoint
        jwt_token (Optional[str]): Current JWT token for authentication
    """
    
    base_url: str
    jwt_token: Optional[str] = None
    
    async def register_did(self, did: str, public_key: str) -> Dict[str, Any]:
        """
        Register a new DID with the system.
        
        Args:
            did (str): Decentralized Identifier to register
            public_key (str): Associated public key for the DID
            
        Returns:
            Dict[str, Any]: Registration response data
            
        Raises:
            httpx.HTTPError: If the registration request fails
        """
        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{self.base_url}/api/v1/auth/register",
                json={"did": did, "public_key": public_key}
            )
            response.raise_for_status()
            return response.json()
    
    async def authenticate(self, did: str, private_key: str) -> None:
        """
        Authenticate with the system using DID and private key.
        
        This method performs the challenge-response authentication flow:
        1. Requests a challenge from the server
        2. Signs the challenge with the provided private key
        3. Submits the signature for verification
        4. Stores the JWT token for subsequent requests
        
        Args:
            did (str): DID to authenticate with
            private_key (str): Private key for signing the challenge
            
        Raises:
            httpx.HTTPError: If any authentication step fails
            ValueError: If the challenge or token is invalid
        """
        async with httpx.AsyncClient() as client:
            # Get challenge
            challenge_resp = await client.post(
                f"{self.base_url}/api/v1/auth/challenge",
                json={"did": did}
            )
            challenge_resp.raise_for_status()
            challenge = challenge_resp.json()["challenge"]
            
            # Sign challenge
            account = Account.from_key(private_key)
            message = encode_defunct(text=challenge)
            signature = account.sign_message(message)
            
            # Verify signature
            verify_resp = await client.post(
                f"{self.base_url}/api/v1/auth/verify",
                json={
                    "did": did,
                    "challenge": challenge,
                    "signature": signature.signature.hex()
                }
            )
            verify_resp.raise_for_status()
            self.jwt_token = verify_resp.json()["token"]
    
    async def create_contract(self, contract_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Create a new contract in the system.
        
        Args:
            contract_data (Dict[str, Any]): Contract details including:
                - title: Contract title
                - description: Contract description
                - consumer_did: DID of the counterparty
                - terms: Contract terms and conditions
                - valid_from: ISO 8601 start date
                - valid_until: ISO 8601 end date
                
        Returns:
            Dict[str, Any]: Created contract data
            
        Raises:
            ValueError: If not authenticated
            httpx.HTTPError: If contract creation fails
        """
        if not self.jwt_token:
            raise ValueError("Not authenticated")
            
        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{self.base_url}/api/v1/contracts",
                json=contract_data,
                headers={"Authorization": f"Bearer {self.jwt_token}"}
            )
            response.raise_for_status()
            return response.json()
    
    async def get_contract(self, contract_id: str) -> Dict[str, Any]:
        """
        Retrieve contract details by ID.
        
        Args:
            contract_id (str): Unique identifier of the contract
            
        Returns:
            Dict[str, Any]: Contract details
            
        Raises:
            ValueError: If not authenticated
            httpx.HTTPError: If contract retrieval fails
        """
        if not self.jwt_token:
            raise ValueError("Not authenticated")
            
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f"{self.base_url}/api/v1/contracts/{contract_id}",
                headers={"Authorization": f"Bearer {self.jwt_token}"}
            )
            response.raise_for_status()
            return response.json()
    
    async def get_contract_events(self, contract_id: str) -> Dict[str, Any]:
        """
        Retrieve blockchain events for a contract.
        
        Args:
            contract_id (str): Unique identifier of the contract
            
        Returns:
            Dict[str, Any]: Contract events from the blockchain
            
        Raises:
            ValueError: If not authenticated
            httpx.HTTPError: If event retrieval fails
        """
        if not self.jwt_token:
            raise ValueError("Not authenticated")
            
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f"{self.base_url}/api/v1/contracts/{contract_id}/events",
                headers={"Authorization": f"Bearer {self.jwt_token}"}
            )
            response.raise_for_status()
            return response.json()
    
    async def verify_contract(self, contract_id: str) -> Dict[str, Any]:
        """
        Verify the integrity and state of a contract on the blockchain.
        
        Args:
            contract_id (str): Unique identifier of the contract
            
        Returns:
            Dict[str, Any]: Verification result including:
                - is_valid: Boolean indicating validity
                - blockchain_state: Current state on chain
                - verification_time: Timestamp of verification
                
        Raises:
            ValueError: If not authenticated
            httpx.HTTPError: If verification fails
        """
        if not self.jwt_token:
            raise ValueError("Not authenticated")
            
        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{self.base_url}/api/v1/contracts/{contract_id}/verify",
                headers={"Authorization": f"Bearer {self.jwt_token}"}
            )
            response.raise_for_status()
            return response.json()

# Example usage and tests
if __name__ == "__main__":
    import asyncio
    
    async def main():
        # Create client
        client = ContractClient("http://localhost:8080")
        
        try:
            # Register DID
            await client.register_did(
                "did:example:123",
                "04a...public_key"
            )
            
            # Authenticate
            await client.authenticate(
                "did:example:123",
                "private_key_here"
            )
            
            # Create contract
            contract_data = {
                "title": "Test Contract",
                "description": "Example contract for testing",
                "consumer_did": "did:example:456",
                "terms": "Test contract terms...",
                "valid_from": datetime.now(timezone.utc).isoformat(),
                "valid_until": datetime.now(timezone.utc).isoformat()
            }
            
            contract = await client.create_contract(contract_data)
            print(f"Created contract: {contract}")
            
            # Verify contract
            verification = await client.verify_contract(contract["id"])
            print(f"Verification result: {verification}")
            
        except httpx.HTTPError as e:
            print(f"HTTP error occurred: {e}")
        except ValueError as e:
            print(f"Validation error: {e}")
        except Exception as e:
            print(f"Unexpected error: {e}")
    
    asyncio.run(main()) 