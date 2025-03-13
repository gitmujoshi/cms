# Web UI Documentation

## Overview
The web interface for the Digital Contract Management System (DCMS) is built using Leptos, a modern Rust framework for building reactive web applications. The UI provides role-specific interfaces for Training Data Providers, Clean Room Providers, Data Consumers, and System Administrators.

## Architecture

### Component Structure
- `app.rs`: Main application component with routing configuration
- `layouts/`: Layout components including the main application layout
- `pages/`: Page components for different routes
- `components/`: Reusable UI components
- `state/`: Application state management
- `api/`: API integration layer

### Authentication
The application implements a secure authentication system with:
- JWT-based authentication
- Role-based access control
- Secure session management
- Persistent login state
- Protected route handling

### Key Components

#### Main Layout (`layouts/main_layout.rs`)
- Responsive navigation header
- Role-based menu items
- Footer with legal links
- Content container with proper spacing

#### Navigation (`components/navbar.rs`)
- Responsive design with mobile menu
- Role-based navigation items
- User menu with logout functionality
- Dynamic route highlighting

#### Dashboard (`pages/dashboard.rs`)
Role-specific dashboards with:
- Welcome message
- Statistics cards
- Quick action buttons
- Loading states
- Error handling

##### Role-Specific Features
1. Training Data Provider
   - Dataset management
   - Contract monitoring
   - Revenue tracking
   
2. Clean Room Provider
   - Clean room status
   - Resource utilization
   - Security monitoring
   
3. Data Consumer
   - Model training status
   - Dataset browsing
   - Contract management
   
4. System Administrator
   - System health monitoring
   - User management
   - Audit logging

#### Stats Component (`components/stats.rs`)
- Reusable statistics card
- Trend indicators (increase/decrease)
- Dynamic value formatting
- Loading state handling

#### Login Page (`pages/login.rs`)
- Email/password authentication
- Form validation
- Error messaging
- Remember me functionality
- Password reset link
- Registration link

## State Management
The application uses Leptos signals for reactive state management:
- `AuthContext`: Manages user authentication state
- `DashboardStats`: Handles dashboard statistics
- Form states: Manages form inputs and validation

## Styling
- Tailwind CSS for utility-first styling
- Responsive design patterns
- Consistent color scheme
- Accessible components

## Security Features
- CSRF protection
- XSS prevention
- Secure cookie handling
- Input sanitization
- Role-based access control

## Error Handling
- Form validation errors
- API error handling
- Network error recovery
- Loading state management
- User-friendly error messages

## Performance Optimizations
- Code splitting
- Lazy loading
- Efficient state updates
- Minimal re-renders
- Asset optimization

## Getting Started

### Prerequisites
```bash
cargo install trunk
cargo install wasm-bindgen-cli
```

### Development
```bash
# Start development server
trunk serve

# Build for production
trunk build --release
```

### Environment Configuration
Create a `.env` file:
```env
API_URL=http://localhost:8080
ENVIRONMENT=development
```

## Contributing
1. Follow the Rust code style guide
2. Write tests for new components
3. Update documentation for changes
4. Submit pull requests with clear descriptions

## Testing
```bash
cargo test
wasm-pack test --chrome
``` 