# Identity and Access Management (IAM) System Design Document

---

## 1. Data Model Design

### 1.1 Realm Structure

This class diagram represents the structure of a Keycloak realm, which is a logical grouping of users, roles, clients, and groups. Realms allow for multi-tenancy and isolation of authentication and authorization data.

[Keycloak Realms](https://www.keycloak.org/docs/latest/server_admin/#realms) | [Keycloak Data Model](https://www.keycloak.org/docs/latest/server_development/#_model)

```mermaid
classDiagram
    class Realm {
        +String name
        +Boolean enabled
        +List~Role~ roles
        +List~Client~ clients
        +List~Group~ groups
    }
    
    class Role {
        +String name
        +String description
        +List~Permission~ permissions
    }
    
    class Client {
        +String clientId
        +Boolean enabled
        +List~String~ redirectUris
        +List~String~ webOrigins
    }
    
    class Group {
        +String name
        +List~Role~ realmRoles
    }
```

### 1.2 User Model

This class diagram details the user model, including attributes such as ID, username, email, enabled status, roles, groups, custom attributes, and support for Decentralized Identifiers (DIDs) and public keys. The addition of DID-related fields and public key storage allows the system to integrate with decentralized identity frameworks, passwordless authentication, and cryptographic login flows.

[Keycloak User Storage](https://www.keycloak.org/docs/latest/server_development/#_user-storage-spi) | [User Federation](https://www.keycloak.org/docs/latest/server_admin/#user-federation) | [Decentralized Identifiers (DIDs) W3C Spec](https://www.w3.org/TR/did-core/) | [Keycloak User Attributes](https://www.keycloak.org/docs/latest/server_admin/#user-attributes)

```mermaid
classDiagram
    class User {
        +String id
        +String username
        +String email
        +Boolean enabled
        +List~Role~ roles
        +List~Group~ groups
        +Map~String, String~ attributes
        +String did                    // Decentralized Identifier
        +List~String~ didMethods       // Supported DID methods (e.g., "key", "web", "ion")
        +Map~String, String~ didDocs   // DID Documents or references
        +Boolean didVerified           // DID verification status
        +String publicKey              // User's public key for cryptographic auth
    }
```

**DID- and Public Key-related attributes:**
- **did:** The user's Decentralized Identifier (DID), e.g., `did:ion:xyz...`
- **didMethods:** List of DID methods supported or used by the user (e.g., `key`, `web`, `ion`).
- **didDocs:** Map containing DID Documents or references to their storage locations.
- **didVerified:** Boolean indicating whether the user's DID has been verified.
- **publicKey:** The user's public key, used for cryptographic authentication (e.g., DID login, passwordless, WebAuthn).

These attributes enable the IAM system to support decentralized identity use cases, passwordless authentication, verifiable credentials, and interoperability with blockchain-based identity systems.

#### 1.2.1 Storing and Retrieving a User's Public Key in Keycloak

You can store a user's public key as a custom attribute in Keycloak, either via the Admin Console or the REST API.

**Example: Storing a Public Key via REST API**

```http
PUT /admin/realms/{realm}/users/{user-id}
Content-Type: application/json
Authorization: Bearer {admin-token}

{
  "attributes": {
    "publicKey": "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkq...\n-----END PUBLIC KEY-----"
  }
}
```

**Retrieving the Public Key:**
- When you fetch the user via the Admin REST API (`GET /admin/realms/{realm}/users/{user-id}`), the `attributes.publicKey` field will contain the stored public key.

**Security Note:**  
- Public keys are not secrets and are safe to store and expose for verification.
- Never store private keys in Keycloak or any server-side system.

---

### 1.3 DID Usage and Integration with Blockchain-based Systems

**Decentralized Identifiers (DIDs)** are globally unique identifiers that are created, owned, and controlled by the user, independent of any centralized authority. DIDs are often anchored on blockchain or distributed ledger systems, providing a tamper-evident and verifiable identity layer.

#### How DIDs are Used in the IAM System

- **User Registration:**  
  During registration, a user can provide a DID or have one generated for them. The DID and its associated DID Document (which contains public keys and service endpoints) are stored as part of the user profile.

- **Authentication:**  
  Users can authenticate using cryptographic proofs (e.g., signing a challenge with their DID private key). The IAM system verifies the signature using the public key from the DID Document, which may be resolved from a blockchain.

- **Verifiable Credentials:**  
  The system can issue, request, or verify [Verifiable Credentials (VCs)](https://www.w3.org/TR/vc-data-model/) associated with a user's DID. These credentials can be used for access control, KYC, or other trust-based workflows.

- **Interoperability:**  
  By supporting multiple DID methods (e.g., `did:key`, `did:web`, `did:ion`), the system can interoperate with various blockchain networks (e.g., Ethereum, Bitcoin, Hyperledger, ION).

#### Example DID-based Authentication Flow

```mermaid
sequenceDiagram
    participant User
    participant IAM
    participant Blockchain

    User->>IAM: Request login (with DID)
    IAM->>User: Challenge (nonce)
    User->>IAM: Signed challenge (using DID private key)
    IAM->>Blockchain: Resolve DID Document & public key
    Blockchain-->>IAM: DID Document
    IAM-->>User: Authentication Success/Failure
```

#### Blockchain Integration

- **DID Anchoring:**  
  DIDs are registered and anchored on a blockchain, ensuring immutability and public verifiability.
- **DID Resolution:**  
  The IAM system can resolve a DID to its DID Document by querying the appropriate blockchain or DID method resolver.
- **Credential Issuance & Verification:**  
  Verifiable credentials can be issued by the IAM system and cryptographically signed using its own DID. Other parties can verify these credentials by resolving the issuer's DID on the blockchain.

#### Benefits

- **Self-sovereign identity:** Users control their own identifiers and credentials.
- **Interoperability:** Works across different platforms and organizations.
- **Security:** Cryptographic proofs and blockchain anchoring provide strong guarantees of authenticity and integrity.
- **Privacy:** Users can selectively disclose information using VCs.

**References:**
- [W3C DID Core Specification](https://www.w3.org/TR/did-core/)
- [W3C Verifiable Credentials Data Model](https://www.w3.org/TR/vc-data-model/)
- [Microsoft ION (DID on Bitcoin)](https://identity.foundation/ion/)
- [Hyperledger Indy](https://www.hyperledger.org/use/hyperledger-indy)
- [Decentralized Identity Foundation](https://identity.foundation/)

---

### 1.4 User Registration and Management

#### 1.4.1 User Registration Flow

This sequence diagram illustrates the process of registering a new user in the system, showing both self-service registration (if enabled) and admin-driven registration.

[Keycloak User Registration](https://www.keycloak.org/docs/latest/server_admin/#user-registration) | [Keycloak REST API: Create User](https://www.keycloak.org/docs-api/21.1.1/rest-api/index.html#_users_resource)

```mermaid
sequenceDiagram
    participant Client
    participant RegistrationEndpoint as /realms/{realm}/protocol/openid-connect/registrations
    participant AdminEndpoint as /admin/realms/{realm}/users
    participant DB

    %% Self-service registration
    Client->>RegistrationEndpoint: POST /realms/{realm}/protocol/openid-connect/registrations (user details)
    RegistrationEndpoint->>DB: Create User
    DB-->>RegistrationEndpoint: User Created
    RegistrationEndpoint-->>Client: Registration Success / Verification Email

    %% Admin-driven registration
    Admin->>AdminEndpoint: POST /admin/realms/{realm}/users (user details)
    AdminEndpoint->>DB: Create User
    DB-->>AdminEndpoint: User Created
    AdminEndpoint-->>Admin: Success/Failure
```

#### 1.4.2 User Management Operations

Keycloak provides a comprehensive set of user management features, including:

- **Create User:**  
  - Admin: `POST /admin/realms/{realm}/users`
  - Self-service: `POST /realms/{realm}/protocol/openid-connect/registrations` (if enabled)
- **Update User:** `PUT /admin/realms/{realm}/users/{id}`
- **Delete User:** `DELETE /admin/realms/{realm}/users/{id}`
- **Enable/Disable User:** `PUT /admin/realms/{realm}/users/{id}`
- **Assign Roles/Groups:**  
  - Roles: `POST /admin/realms/{realm}/users/{id}/role-mappings/realm`
  - Groups: `PUT /admin/realms/{realm}/users/{id}/groups/{groupId}`
- **Password Reset:** `PUT /admin/realms/{realm}/users/{id}/reset-password`
- **Email Verification:** `PUT /admin/realms/{realm}/users/{id}/send-verify-email`

These operations can be performed via the Keycloak Admin Console or programmatically using the [Keycloak Admin REST API](https://www.keycloak.org/docs-api/21.1.1/rest-api/index.html#_users_resource).

**References:**
- [Keycloak User Management](https://www.keycloak.org/docs/latest/server_admin/#user-management)
- [Keycloak Admin REST API: Users](https://www.keycloak.org/docs-api/21.1.1/rest-api/index.html#_users_resource)

#### 1.4.3 Sample Script: Registration and Authentication with Public Key

**Registration (Python Example):**
```python
import requests

KEYCLOAK_URL = "https://iam.example.com"
REALM = "your-realm"
ADMIN_TOKEN = "..."  # Obtain via client credentials grant or admin login

def register_user(username, email, public_key_pem):
    url = f"{KEYCLOAK_URL}/admin/realms/{REALM}/users"
    headers = {
        "Authorization": f"Bearer {ADMIN_TOKEN}",
        "Content-Type": "application/json"
    }
    data = {
        "username": username,
        "email": email,
        "enabled": True,
        "attributes": {
            "publicKey": public_key_pem
        }
    }
    resp = requests.post(url, json=data, headers=headers)
    if resp.status_code == 201:
        print("User registered successfully!")
    else:
        print("Error:", resp.status_code, resp.text)
```

**Authentication (Python Example):**
```python
import requests
from cryptography.hazmat.primitives import serialization, hashes
from cryptography.hazmat.primitives.asymmetric import padding
import base64

def get_user_public_key(username):
    url = f"{KEYCLOAK_URL}/admin/realms/{REALM}/users"
    headers = {"Authorization": f"Bearer {ADMIN_TOKEN}"}
    params = {"username": username}
    resp = requests.get(url, headers=headers, params=params)
    users = resp.json()
    if users:
        return users[0]["attributes"]["publicKey"][0]
    return None

def verify_signature(public_key_pem, challenge, signature_b64):
    public_key = serialization.load_pem_public_key(public_key_pem.encode())
    signature = base64.b64decode(signature_b64)
    try:
        public_key.verify(
            signature,
            challenge.encode(),
            padding.PKCS1v15(),
            hashes.SHA256()
        )
        return True
    except Exception as e:
        print("Verification failed:", e)
        return False
```

---

## 2. Authentication Design

### 2.1 Authentication Flow

This sequence diagram illustrates the authentication process using Keycloak as the identity provider, showing the actual endpoints involved. The client initiates authentication via the OpenID Connect authorization endpoint, receives an authorization code, and exchanges it for tokens at the token endpoint.

[Keycloak Authentication Flows](https://www.keycloak.org/docs/latest/server_admin/#authentication-flows) | [JWT Introduction](https://jwt.io/introduction/)

```mermaid
sequenceDiagram
    participant Client
    participant AuthEndpoint as /realms/{realm}/protocol/openid-connect/auth
    participant TokenEndpoint as /realms/{realm}/protocol/openid-connect/token
    participant DB
    
    Client->>AuthEndpoint: GET /realms/{realm}/protocol/openid-connect/auth (login)
    AuthEndpoint->>DB: Validate Credentials
    DB-->>AuthEndpoint: Credential Status
    AuthEndpoint-->>Client: Auth code / login page

    Client->>TokenEndpoint: POST /realms/{realm}/protocol/openid-connect/token (exchange code)
    TokenEndpoint-->>Client: JWT Token
```

### 2.2 Authentication Methods

- Username/Password
- OAuth 2.0
- OpenID Connect
- Social Login (configurable)
- LDAP (configurable)

## 3. Authorization Design

### 3.1 Authorization Flow

This sequence diagram shows how authorization is handled using actual Keycloak endpoints. The client requests an access token from the token endpoint, uses it to access a protected resource, and the resource server validates the token (optionally using the userinfo endpoint).

[OAuth 2.0 Authorization Framework](https://datatracker.ietf.org/doc/html/rfc6749) | [Keycloak Authorization Services](https://www.keycloak.org/docs/latest/authorization_services/)

```mermaid
sequenceDiagram
    participant Client
    participant TokenEndpoint as /realms/{realm}/protocol/openid-connect/token
    participant ResourceAPI as /resource
    participant UserInfoEndpoint as /realms/{realm}/protocol/openid-connect/userinfo

    Client->>TokenEndpoint: POST /realms/{realm}/protocol/openid-connect/token (get access token)
    TokenEndpoint-->>Client: Access Token
    Client->>ResourceAPI: GET /resource (with access token)
    ResourceAPI->>UserInfoEndpoint: GET /realms/{realm}/protocol/openid-connect/userinfo (validate token)
    UserInfoEndpoint-->>ResourceAPI: User info / token status
    ResourceAPI-->>Client: Resource Access
```

### 3.2 Authorization Models

- Role-Based Access Control (RBAC)
- Group-Based Permissions
- Fine-Grained Access Control
- Token-Based Authorization

## 4. Integration Design

### 4.1 API Design

This sequence diagram outlines the main Keycloak REST API endpoints and their interactions with the client. The client authenticates, refreshes tokens, retrieves user info, and accesses admin endpoints. Each interaction is shown as a request-response pair with the actual Keycloak endpoint.

[Keycloak REST API](https://www.keycloak.org/docs-api/21.1.1/rest-api/index.html) | [OpenID Connect Discovery](https://openid.net/specs/openid-connect-discovery-1_0.html)

**Key Endpoints:**
- `/realms/{realm}/protocol/openid-connect/auth` — Authorization endpoint (user login)
- `/realms/{realm}/protocol/openid-connect/token` — Token endpoint (get/refresh tokens)
- `/realms/{realm}/protocol/openid-connect/userinfo` — User info endpoint
- `/admin/realms/{realm}/users` — Admin user management endpoint

```mermaid
sequenceDiagram
    participant Client
    participant AuthEndpoint as /realms/{realm}/protocol/openid-connect/auth
    participant TokenEndpoint as /realms/{realm}/protocol/openid-connect/token
    participant UserInfoEndpoint as /realms/{realm}/protocol/openid-connect/userinfo
    participant AdminEndpoint as /admin/realms/{realm}/users

    Client->>AuthEndpoint: GET /realms/{realm}/protocol/openid-connect/auth (login)
    AuthEndpoint-->>Client: Auth code / login page

    Client->>TokenEndpoint: POST /realms/{realm}/protocol/openid-connect/token (exchange code/refresh token)
    TokenEndpoint-->>Client: Access/Refresh token

    Client->>UserInfoEndpoint: GET /realms/{realm}/protocol/openid-connect/userinfo (with token)
    UserInfoEndpoint-->>Client: User info

    Client->>AdminEndpoint: Admin operations (with admin token)
    AdminEndpoint-->>Client: Admin response
```

### 4.2 Protocol Support

- OAuth 2.0
- OpenID Connect
- SAML 2.0
- LDAP

## 5. Security Design

### 5.1 Token Design

This class diagram shows the structure of tokens used in the system. JWT tokens consist of a header, payload, and signature, and provide methods for validation and claim retrieval. The Token class represents access and refresh tokens, their expiry, and type.

[JSON Web Token (JWT) Specification](https://datatracker.ietf.org/doc/html/rfc7519) | [Keycloak Token Types](https://www.keycloak.org/docs/latest/server_admin/#_tokens)

```mermaid
classDiagram
    class JWT {
        +String header
        +String payload
        +String signature
        +validate()
        +getClaims()
    }
    
    class Token {
        +String accessToken
        +String refreshToken
        +Integer expiresIn
        +String tokenType
    }
```

**Token Management Endpoints:**
- **Token Introspection:** `POST /realms/{realm}/protocol/openid-connect/token/introspect`
- **Token Revocation:** `POST /realms/{realm}/protocol/openid-connect/revoke`
- **Logout:** `POST /realms/{realm}/protocol/openid-connect/logout`

**Example Token Introspection Flow:**
```mermaid
sequenceDiagram
    participant ResourceServer
    participant IntrospectEndpoint as /realms/{realm}/protocol/openid-connect/token/introspect

    ResourceServer->>IntrospectEndpoint: POST (token)
    IntrospectEndpoint-->>ResourceServer: Token status (active/inactive, claims)
```

### 5.2 Security Features

- Password Policies
- Brute Force Protection
- Session Management
- Audit Logging
- Token Validation

## 6. Backup and Recovery Design

### 6.1 Backup Strategy

This flowchart demonstrates the backup strategy, including daily backups of the database, configuration, and realm exports, all stored in a backup storage location. This ensures disaster recovery and data integrity.

[Keycloak Backup and Restore](https://www.keycloak.org/docs/latest/server_admin/#_backup_restore) | [Database Backup Best Practices](https://www.postgresql.org/docs/current/backup.html)

**Key Backup/Restore Endpoints and Commands:**
- **Export Realm (REST):** `POST /admin/realms/{realm}/partial-export`
- **Import Realm (REST):** `POST /admin/realms/{realm}/partial-import`
- **Export/Import (CLI):** `bin/kc.sh export` and `bin/kc.sh import`

```mermaid
graph TD
    A[Daily Backup] --> B[Database Backup]
    A --> C[Configuration Backup]
    A --> D["Realm Export<br/>/admin/realms/{realm}/partial-export"]
    B --> E[Backup Storage]
    C --> E
    D --> E
```

### 6.2 Recovery Procedures

- Database Restoration
- Configuration Restoration
- Realm Import (`/admin/realms/{realm}/partial-import` or `kc.sh import`)
- Service Recovery

## 7. Performance Design

### 7.1 Caching Design

This flowchart shows the caching strategy, where different types of caches (token, session, permission, realm) are stored in Redis. This improves performance and scalability by reducing database load.

[Keycloak Caching](https://www.keycloak.org/docs/latest/server_installation/#cache) | [Redis Documentation](https://redis.io/docs/)

```mermaid
graph TD
    A[Token Cache] --> B[Redis]
    C[Session Cache] --> B
    D[Permission Cache] --> B
    E[Realm Cache] --> B
```

### 7.2 Scaling Design

- Horizontal Scaling
- Load Balancing
- Database Sharding
- Cache Distribution

## 8. Future Design Considerations

### 8.1 Planned Features

- Multi-Factor Authentication
- Social Login Integration
- Custom Authentication Flows
- Advanced Authorization Policies
- Audit Logging Enhancements

### 8.2 Design Improvements

- Microservices Architecture
- Event-Driven Design
- API Gateway Integration
- Service Mesh Implementation 
