# Registration Component Documentation

## Overview
The Registration component (`pages/registration.rs`) handles the participant registration process for the Digital Contract Management System. It provides a secure and user-friendly interface for organizations to register as different types of participants.

## Component Structure

### Data Models

#### RegistrationForm
```rust
pub struct RegistrationForm {
    organization_name: String,    // Length: 3-100 chars
    email: String,               // Valid email format
    password: String,            // Min length: 8 chars
    confirm_password: String,    // Must match password
    participant_type: Vec<ParticipantType>,  // At least 1 type
    contact_info: ContactInfo,   // Contact details
    terms_accepted: bool,        // Must be true
}
```

#### ParticipantType
```rust
pub enum ParticipantType {
    TrainingDataProvider,    // Organizations providing training data
    CleanRoomProvider,       // Organizations hosting clean rooms
    DataConsumer,            // Organizations consuming training data
    SystemAdministrator,     // System administrators
}
```

#### ContactInfo
```rust
pub struct ContactInfo {
    full_name: String,    // Length: 3-100 chars
    phone: String,        // Valid phone format
    address: String,      // Length: 5-200 chars
}
```

## Features

### Form Validation
- Client-side validation using the `validator` crate
- Required field validation
- Email format validation
- Password strength requirements
- Phone number format validation
- Address validation
- Terms acceptance requirement

### Error Handling
- Form-level error collection
- Field-specific error messages
- API error handling and display
- User-friendly error messages

### Security
- Password confirmation
- Terms and conditions acceptance
- Email verification flow
- CSRF protection
- Input sanitization

### UI/UX
- Responsive design
- Clear error indicators
- Progressive form completion
- Accessible form controls
- Loading states
- Success/failure notifications

## Usage

### Basic Implementation
```rust
use leptos::*;
use crate::pages::registration::RegistrationPage;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <RegistrationPage />
    }
}
```

### Custom Configuration
```rust
// Example of customizing validation rules
#[derive(Validate)]
pub struct CustomRegistrationForm {
    #[validate(length(min = 5, max = 150))]
    organization_name: String,
    // ... other fields
}
```

## API Integration

### Registration Endpoint
```rust
async fn register_participant(form: RegistrationForm) -> Result<(), String> {
    // POST request to /api/register
    // Returns success or error message
}
```

### Response Handling
- Success: Redirect to login page with verification notice
- Error: Display error message in form
- Network issues: Show connection error

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_validation() {
        // Test validation rules
    }

    #[test]
    fn test_participant_type_selection() {
        // Test role selection
    }
}
```

### Integration Tests
```rust
#[wasm_bindgen_test]
async fn test_registration_flow() {
    // Test complete registration process
}
```

## Styling
The component uses Tailwind CSS classes for styling:
- Responsive layout
- Form element styling
- Error state styling
- Button states
- Loading animations

## Accessibility
- ARIA labels
- Keyboard navigation
- Error announcements
- Focus management
- High contrast support

## Performance
- Efficient form state management
- Optimized validation
- Minimal re-renders
- Lazy loading of heavy components

## Future Enhancements
- Multi-step registration process
- OAuth integration
- Organization verification
- Custom role definitions
- Enhanced security measures 