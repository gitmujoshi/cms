# UI Components Documentation

## Core Components

### 1. Navigation Bar (`components/navbar.rs`)

The navigation bar provides the main navigation interface for the application.

#### Features
- Responsive design with mobile menu
- Role-based navigation items
- Authentication state handling
- Dynamic route highlighting

#### Usage
```rust
use crate::components::Navbar;

#[component]
pub fn Layout() -> impl IntoView {
    view! {
        <Navbar />
        // ... rest of layout
    }
}
```

#### Props
None - Uses global authentication context

#### States
- Mobile menu state
- Authentication state
- Current route state

### 2. Stats Card (`components/stats.rs`)

Reusable component for displaying statistics with trend indicators.

#### Features
- Value display with formatting
- Trend indicators (increase/decrease)
- Percentage change display
- Detail link

#### Usage
```rust
use crate::components::StatsCard;

#[component]
pub fn Dashboard() -> impl IntoView {
    view! {
        <StatsCard
            title="Active Contracts"
            value=Some("42".to_string())
            change=Some(10)
            trend="increase"
        />
    }
}
```

#### Props
- `title: &'static str` - Card title
- `value: Option<String>` - Main statistic value
- `change: Option<i32>` - Percentage change
- `trend: &'static str` - Trend direction ("increase" or "decrease")

## Page Components

### 1. Dashboard Page (`pages/dashboard.rs`)

Role-specific dashboard displaying relevant statistics and actions.

#### Features
- Role-based content
- Statistics display
- Quick actions
- Loading states
- Error handling

#### Sections
1. Welcome Message
   - Organization name
   - Role-specific message

2. Statistics Section
   - Role-specific stats cards
   - Dynamic loading
   - Error handling

3. Quick Actions
   - Role-based action buttons
   - Icon integration
   - Action routing

#### States
```rust
#[derive(Debug, Clone)]
struct DashboardStats {
    active_contracts: u32,
    datasets: u32,
    revenue: f64,
    // ... other stats
}
```

### 2. Login Page (`pages/login.rs`)

Handles user authentication with form validation.

#### Features
- Email/password form
- Validation
- Error messaging
- Remember me
- Password reset
- Registration link

#### Form Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct LoginForm {
    #[validate(email)]
    email: String,
    
    #[validate(length(min = 8))]
    password: String,
}
```

#### States
- Form state
- Validation errors
- Loading state
- Authentication state

## Layout Components

### Main Layout (`layouts/main_layout.rs`)

Primary layout wrapper for the application.

#### Features
- Navigation integration
- Content container
- Footer
- Responsive design

#### Usage
```rust
use crate::layouts::MainLayout;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <MainLayout>
                // ... routes
            </MainLayout>
        </Router>
    }
}
```

#### Structure
1. Navigation Section
   - Navbar component
   - Authentication state

2. Content Section
   - Children rendering
   - Padding/margin
   - Max width constraints

3. Footer Section
   - Legal links
   - Copyright
   - Additional links

## State Management

### Authentication Context (`state/auth.rs`)

Manages global authentication state.

#### Features
- User information
- Token management
- Role-based access
- Local storage persistence

#### Structure
```rust
#[derive(Debug, Clone)]
pub struct AuthContext {
    user: Option<User>,
    token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub organization_name: String,
    pub role: Role,
}
```

#### Methods
- `login(email: &str, password: &str) -> Result<(), String>`
- `logout() -> Result<(), String>`
- `is_authenticated() -> bool`
- `role() -> Option<&Role>`

## Styling Guidelines

### CSS Classes

#### Layout Classes
- `min-h-screen` - Full height layout
- `max-w-7xl` - Maximum content width
- `mx-auto` - Center alignment
- `px-4 sm:px-6 lg:px-8` - Responsive padding

#### Component Classes
- `bg-white` - White background
- `shadow` - Box shadow
- `rounded-lg` - Rounded corners
- `text-gray-900` - Primary text color

#### Interactive Classes
- `hover:text-gray-700` - Hover state
- `focus:ring-2` - Focus ring
- `active:bg-gray-100` - Active state

## Best Practices

### Component Development
1. Use TypeScript for prop types
2. Implement error boundaries
3. Handle loading states
4. Include accessibility attributes

### State Management
1. Minimize global state
2. Use local state when possible
3. Implement proper cleanup
4. Handle side effects

### Performance
1. Lazy load components
2. Optimize re-renders
3. Use memo where appropriate
4. Clean up subscriptions

### Accessibility
1. Include ARIA labels
2. Ensure keyboard navigation
3. Maintain color contrast
4. Support screen readers 