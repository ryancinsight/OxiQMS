// QMS Setup Wizard JavaScript
// Medical Device Quality Management System Setup Interface
// FDA 21 CFR Part 820, ISO 13485, ISO 14971 Compliant

console.log('🚀 QMS Setup JavaScript loading...');

class QMSSetupWizard {
    constructor() {
        console.log('🔧 QMSSetupWizard constructor called');
        this.currentStep = 1;
        this.totalSteps = 3;
        this.setupData = {};

        try {
            this.initializeEventListeners();
            this.updateProgress();
            console.log('✅ QMSSetupWizard initialized successfully');
        } catch (error) {
            console.error('❌ Error initializing QMSSetupWizard:', error);
        }
    }

    initializeEventListeners() {
        console.log('Initializing event listeners...');

        // Navigation buttons
        const nextButton = document.getElementById('next-button');
        const backButton = document.getElementById('back-button');
        const initButton = document.getElementById('initialize-button');
        const browseButton = document.getElementById('browse-directory');

        if (nextButton) {
            console.log('✅ Next button found, attaching event listener');
            nextButton.addEventListener('click', (e) => {
                e.preventDefault();
                console.log('🔥 Next button clicked!');
                this.nextStep();
            });
            console.log('✅ Next button event listener attached');
        } else {
            console.error('❌ Next button not found in DOM');
        }

        if (backButton) {
            backButton.addEventListener('click', (e) => {
                e.preventDefault();
                console.log('Back button clicked');
                this.previousStep();
            });
        }

        if (initButton) {
            initButton.addEventListener('click', (e) => {
                e.preventDefault();
                console.log('Initialize button clicked');
                this.initializeQMS();
            });
        }

        if (browseButton) {
            console.log('✅ Browse button found, attaching event listener');
            browseButton.addEventListener('click', (e) => {
                e.preventDefault();
                console.log('🔥 Browse button clicked!');
                this.browseDirectory();
            });
            console.log('✅ Browse button event listener attached');
        } else {
            console.error('❌ Browse button not found in DOM');
        }

        // Form validation with error checking
        const projectDirectory = document.getElementById('project-directory');
        const projectName = document.getElementById('project-name');
        const adminPassword = document.getElementById('admin-password');
        const adminPasswordConfirm = document.getElementById('admin-password-confirm');
        const adminEmail = document.getElementById('admin-email');

        if (projectDirectory) {
            projectDirectory.addEventListener('change', () => this.validateDirectory());
            projectDirectory.addEventListener('input', () => this.validateStep1Fields());
            projectDirectory.addEventListener('blur', () => this.validateDirectory());
        }

        if (projectName) {
            projectName.addEventListener('input', () => this.validateStep1Fields());
            projectName.addEventListener('blur', () => this.validateStep1Fields());
        }

        if (adminPassword) {
            adminPassword.addEventListener('input', () => this.validateStep2Fields());
            adminPassword.addEventListener('blur', () => this.validatePasswords());
        }

        if (adminPasswordConfirm) {
            adminPasswordConfirm.addEventListener('input', () => this.validateStep2Fields());
            adminPasswordConfirm.addEventListener('blur', () => this.validatePasswords());
        }

        if (adminEmail) {
            adminEmail.addEventListener('input', () => this.validateStep2Fields());
            adminEmail.addEventListener('blur', () => this.validateEmail());
        }

        const adminUsername = document.getElementById('admin-username');
        if (adminUsername) {
            adminUsername.addEventListener('input', () => this.validateStep2Fields());
        }

        console.log('Event listeners initialized successfully');
    }

    async nextStep() {
        console.log(`Attempting to advance from step ${this.currentStep}`);

        try {
            if (this.currentStep === 1) {
                console.log('Validating step 1...');
                const isValid = await this.validateStep1();
                console.log('Step 1 validation result:', isValid);
                if (!isValid) {
                    console.log('Step 1 validation failed, not advancing');
                    return;
                }
            } else if (this.currentStep === 2) {
                console.log('Validating step 2...');
                const isValid = this.validateStep2();
                console.log('Step 2 validation result:', isValid);
                if (!isValid) {
                    console.log('Step 2 validation failed, not advancing');
                    return;
                }
                this.populateSummary();
            }

            if (this.currentStep < this.totalSteps) {
                this.currentStep++;
                console.log(`Advanced to step ${this.currentStep}`);
                this.updateStepDisplay();
                this.updateProgress();
            }
        } catch (error) {
            console.error('Error in nextStep:', error);
            this.showError('An error occurred while advancing to the next step: ' + error.message);
        }
    }

    previousStep() {
        if (this.currentStep > 1) {
            this.currentStep--;
            this.updateStepDisplay();
            this.updateProgress();
        }
    }

    updateStepDisplay() {
        console.log(`Updating display for step ${this.currentStep}`);

        // Hide all forms
        for (let i = 1; i <= this.totalSteps; i++) {
            const form = document.getElementById(`step-${i}-form`);
            const step = document.getElementById(`step-${i}`);

            if (form) {
                form.classList.add('hidden');
            } else {
                console.warn(`Form step-${i}-form not found`);
            }

            if (step) {
                step.classList.remove('active', 'completed');
            } else {
                console.warn(`Step indicator step-${i} not found`);
            }
        }

        // Show current form
        const currentForm = document.getElementById(`step-${this.currentStep}-form`);
        const currentStep = document.getElementById(`step-${this.currentStep}`);

        if (currentForm) {
            currentForm.classList.remove('hidden');
        } else {
            console.error(`Current form step-${this.currentStep}-form not found`);
        }

        if (currentStep) {
            currentStep.classList.add('active');
        } else {
            console.error(`Current step indicator step-${this.currentStep} not found`);
        }

        // Mark completed steps
        for (let i = 1; i < this.currentStep; i++) {
            const completedStep = document.getElementById(`step-${i}`);
            if (completedStep) {
                completedStep.classList.add('completed');
            }
        }

        // Update navigation buttons
        const backButton = document.getElementById('back-button');
        const nextButton = document.getElementById('next-button');
        const initButton = document.getElementById('initialize-button');

        if (backButton) {
            if (this.currentStep === 1) {
                backButton.classList.add('hidden');
            } else {
                backButton.classList.remove('hidden');
            }
        }

        if (nextButton && initButton) {
            if (this.currentStep === this.totalSteps) {
                nextButton.classList.add('hidden');
                initButton.classList.remove('hidden');
            } else {
                nextButton.classList.remove('hidden');
                initButton.classList.add('hidden');
            }
        }

        // Clear any error messages when changing steps
        this.hideError();
        this.clearAllFieldHighlights();

        console.log(`Step display updated successfully for step ${this.currentStep}`);
    }

    updateProgress() {
        const progress = (this.currentStep / this.totalSteps) * 100;
        const progressFill = document.getElementById('progress-fill');
        if (progressFill) {
            progressFill.style.width = `${progress}%`;
        }
    }

    validateStep1Fields() {
        // Real-time validation for step 1 fields
        const projectName = document.getElementById('project-name').value.trim();
        const directory = document.getElementById('project-directory').value.trim();

        // Clear previous highlights
        this.highlightField('project-name', false);
        this.highlightField('project-directory', false);

        let hasErrors = false;

        // Validate project name
        if (projectName.length > 0 && projectName.length < 3) {
            this.highlightField('project-name', true);
            hasErrors = true;
        }

        // If both fields have content, clear errors
        if (projectName.length >= 3 && directory.length > 0 && !hasErrors) {
            this.hideError();
        }
    }

    validateStep2Fields() {
        // Real-time validation for step 2 fields
        const username = document.getElementById('admin-username').value.trim();
        const email = document.getElementById('admin-email').value.trim();
        const password = document.getElementById('admin-password').value;
        const confirmPassword = document.getElementById('admin-password-confirm').value;

        // Clear previous highlights
        this.highlightField('admin-username', false);
        this.highlightField('admin-email', false);
        this.highlightField('admin-password', false);
        this.highlightField('admin-password-confirm', false);

        let hasErrors = false;

        // Validate username
        if (username.length > 0 && username.length < 3) {
            this.highlightField('admin-username', true);
            hasErrors = true;
        }

        // Validate email
        if (email.length > 0 && !this.isValidEmail(email)) {
            this.highlightField('admin-email', true);
            hasErrors = true;
        }

        // Validate password
        if (password.length > 0 && password.length < 8) {
            this.highlightField('admin-password', true);
            hasErrors = true;
        }

        // Validate password confirmation
        if (confirmPassword.length > 0 && password !== confirmPassword) {
            this.highlightField('admin-password', true);
            this.highlightField('admin-password-confirm', true);
            hasErrors = true;
        }

        // If all fields are valid, clear errors
        if (!hasErrors && username.length >= 3 && this.isValidEmail(email) &&
            password.length >= 8 && password === confirmPassword) {
            this.hideError();
        }
    }

    async browseDirectory() {
        console.log('Browse directory clicked');

        // Check if the File System Access API is available (Chrome 86+)
        if ('showDirectoryPicker' in window) {
            try {
                const directoryHandle = await window.showDirectoryPicker();
                const directoryPath = directoryHandle.name;
                document.getElementById('project-directory').value = directoryPath;
                console.log('Directory selected via File System Access API:', directoryPath);
                await this.validateDirectory();
                return;
            } catch (error) {
                console.log('User cancelled directory picker or error occurred:', error);
                // Fall through to alternative method
            }
        }

        // Fallback: Enhanced prompt with better UX
        this.showDirectoryInputDialog();
    }

    showDirectoryInputDialog() {
        // Create a more user-friendly directory input dialog
        const modal = document.createElement('div');
        modal.style.cssText = `
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: rgba(0, 0, 0, 0.5);
            display: flex;
            justify-content: center;
            align-items: center;
            z-index: 1000;
        `;

        const dialog = document.createElement('div');
        dialog.style.cssText = `
            background: white;
            padding: 2rem;
            border-radius: 8px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
            max-width: 500px;
            width: 90%;
        `;

        dialog.innerHTML = `
            <h3 style="margin-top: 0; color: #2c3e50;">Select Project Directory</h3>
            <p style="color: #7f8c8d; margin-bottom: 1rem;">
                Enter the full path where you want to create your QMS project.
                The directory will be created if it doesn't exist.
            </p>
            <div style="margin-bottom: 1rem;">
                <label style="display: block; margin-bottom: 0.5rem; font-weight: 600;">Directory Path:</label>
                <input type="text" id="directory-input" placeholder="e.g., C:\\Projects\\MyQMS or /home/user/qms-project"
                       style="width: 100%; padding: 0.75rem; border: 1px solid #ddd; border-radius: 4px; font-size: 1rem;">
            </div>
            <div style="margin-bottom: 1rem;">
                <p style="font-size: 0.9rem; color: #7f8c8d; margin: 0;">
                    <strong>Examples:</strong><br>
                    Windows: <code>C:\\Users\\YourName\\Documents\\QMS</code><br>
                    macOS/Linux: <code>/Users/YourName/Documents/QMS</code>
                </p>
            </div>
            <div style="display: flex; gap: 1rem; justify-content: flex-end;">
                <button id="cancel-directory" style="padding: 0.75rem 1.5rem; border: 1px solid #ddd; background: white; border-radius: 4px; cursor: pointer;">
                    Cancel
                </button>
                <button id="confirm-directory" style="padding: 0.75rem 1.5rem; background: #3498db; color: white; border: none; border-radius: 4px; cursor: pointer;">
                    Select Directory
                </button>
            </div>
        `;

        modal.appendChild(dialog);
        document.body.appendChild(modal);

        const input = dialog.querySelector('#directory-input');
        const cancelBtn = dialog.querySelector('#cancel-directory');
        const confirmBtn = dialog.querySelector('#confirm-directory');

        // Focus the input
        setTimeout(() => input.focus(), 100);

        // Handle cancel
        cancelBtn.addEventListener('click', () => {
            document.body.removeChild(modal);
        });

        // Handle confirm
        confirmBtn.addEventListener('click', async () => {
            const directory = input.value.trim();
            if (directory) {
                document.getElementById('project-directory').value = directory;
                document.body.removeChild(modal);
                await this.validateDirectory();
            } else {
                input.style.borderColor = '#e74c3c';
                input.focus();
            }
        });

        // Handle Enter key
        input.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                confirmBtn.click();
            }
        });

        // Handle Escape key
        modal.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                cancelBtn.click();
            }
        });
    }

    async validateDirectory() {
        const directoryInput = document.getElementById('project-directory');
        const directory = directoryInput.value.trim();
        
        if (!directory) {
            this.showError('Please select a directory for your QMS project.');
            return false;
        }

        try {
            const response = await fetch('/api/setup/validate-directory', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ directory: directory })
            });

            const result = await response.json();
            
            if (result.valid) {
                this.hideError();
                this.setupData.directory = directory;
                return true;
            } else {
                let errorMsg = 'Invalid directory: ';
                if (!result.exists) {
                    errorMsg += 'Directory does not exist and cannot be created.';
                } else if (!result.writable) {
                    errorMsg += 'Directory is not writable.';
                } else if (!result.empty) {
                    errorMsg += 'Directory is not empty.';
                }
                this.showError(errorMsg);
                return false;
            }
        } catch (error) {
            this.showError('Failed to validate directory: ' + error.message);
            return false;
        }
    }

    async validateStep1() {
        console.log('Validating step 1 fields...');
        const projectName = document.getElementById('project-name').value.trim();
        const directory = document.getElementById('project-directory').value.trim();

        // Clear any existing errors first
        this.hideError();

        // Validate project name
        if (!projectName) {
            this.showError('Please enter a project name.');
            this.highlightField('project-name', true);
            return false;
        }

        if (projectName.length < 3) {
            this.showError('Project name must be at least 3 characters long.');
            this.highlightField('project-name', true);
            return false;
        }

        // Validate directory
        if (!directory) {
            this.showError('Please select a project directory.');
            this.highlightField('project-directory', true);
            return false;
        }

        // Clear field highlights if validation passes
        this.highlightField('project-name', false);
        this.highlightField('project-directory', false);

        // Store valid data
        this.setupData.projectName = projectName;

        // Validate directory with server
        console.log('Validating directory with server...');
        const directoryValid = await this.validateDirectory();
        console.log('Directory validation result:', directoryValid);

        return directoryValid;
    }

    validateStep2() {
        console.log('Validating step 2 fields...');
        const username = document.getElementById('admin-username').value.trim();
        const email = document.getElementById('admin-email').value.trim();
        const password = document.getElementById('admin-password').value;
        const confirmPassword = document.getElementById('admin-password-confirm').value;

        // Clear any existing errors first
        this.hideError();
        this.clearAllFieldHighlights();

        // Validate username
        if (!username) {
            this.showError('Please enter an administrator username.');
            this.highlightField('admin-username', true);
            return false;
        }

        if (username.length < 3) {
            this.showError('Username must be at least 3 characters long.');
            this.highlightField('admin-username', true);
            return false;
        }

        // Validate email
        if (!email) {
            this.showError('Please enter an administrator email.');
            this.highlightField('admin-email', true);
            return false;
        }

        if (!this.isValidEmail(email)) {
            this.showError('Please enter a valid email address.');
            this.highlightField('admin-email', true);
            return false;
        }

        // Validate password
        if (!password) {
            this.showError('Please enter a password.');
            this.highlightField('admin-password', true);
            return false;
        }

        if (password.length < 8) {
            this.showError('Password must be at least 8 characters long.');
            this.highlightField('admin-password', true);
            return false;
        }

        // Validate password confirmation
        if (!confirmPassword) {
            this.showError('Please confirm your password.');
            this.highlightField('admin-password-confirm', true);
            return false;
        }

        if (password !== confirmPassword) {
            this.showError('Passwords do not match.');
            this.highlightField('admin-password', true);
            this.highlightField('admin-password-confirm', true);
            return false;
        }

        // Store valid data
        this.setupData.adminUsername = username;
        this.setupData.adminEmail = email;
        this.setupData.adminPassword = password;

        console.log('Step 2 validation passed');
        this.hideError();
        return true;
    }

    validatePasswords() {
        const password = document.getElementById('admin-password').value;
        const confirmPassword = document.getElementById('admin-password-confirm').value;

        // Clear previous highlights
        this.highlightField('admin-password', false);
        this.highlightField('admin-password-confirm', false);

        if (password.length > 0 && password.length < 8) {
            this.showFieldError('admin-password', 'Password must be at least 8 characters long.');
            return false;
        }

        if (confirmPassword.length > 0 && password !== confirmPassword) {
            this.highlightField('admin-password', true);
            this.highlightField('admin-password-confirm', true);
            this.showError('Passwords do not match.');
            return false;
        }

        if (password.length >= 8 && confirmPassword.length > 0 && password === confirmPassword) {
            this.hideError();
            return true;
        }

        return password.length === 0; // Valid if empty (not required for real-time validation)
    }

    validateEmail() {
        const email = document.getElementById('admin-email').value.trim();

        this.highlightField('admin-email', false);

        if (email && !this.isValidEmail(email)) {
            this.showFieldError('admin-email', 'Please enter a valid email address.');
            return false;
        } else if (email && this.isValidEmail(email)) {
            this.hideError();
            return true;
        }

        return email.length === 0; // Valid if empty (not required for real-time validation)
    }

    isValidEmail(email) {
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        return emailRegex.test(email);
    }

    populateSummary() {
        document.getElementById('summary-directory').textContent = this.setupData.directory;
        document.getElementById('summary-project-name').textContent = this.setupData.projectName;
        document.getElementById('summary-admin-username').textContent = this.setupData.adminUsername;
        document.getElementById('summary-admin-email').textContent = this.setupData.adminEmail;
    }

    async initializeQMS() {
        const initButton = document.getElementById('initialize-button');
        initButton.disabled = true;
        initButton.textContent = 'Initializing...';

        try {
            const response = await fetch('/api/setup/initialize', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    directory: this.setupData.directory,
                    project_name: this.setupData.projectName,
                    admin_username: this.setupData.adminUsername,
                    admin_email: this.setupData.adminEmail,
                    admin_password: this.setupData.adminPassword
                })
            });

            const result = await response.json();
            
            if (result.success) {
                this.showSuccess('QMS system initialized successfully! Redirecting to main application...');
                
                // Redirect to main application after a short delay
                setTimeout(() => {
                    window.location.href = '/';
                }, 2000);
            } else {
                this.showError('Failed to initialize QMS: ' + (result.error || result.message || 'Unknown error'));
                initButton.disabled = false;
                initButton.textContent = 'Initialize QMS';
            }
        } catch (error) {
            this.showError('Failed to initialize QMS: ' + error.message);
            initButton.disabled = false;
            initButton.textContent = 'Initialize QMS';
        }
    }

    showError(message) {
        const errorDiv = document.getElementById('error-message');
        errorDiv.textContent = message;
        errorDiv.style.display = 'block';
        
        const successDiv = document.getElementById('success-message');
        successDiv.style.display = 'none';
    }

    showSuccess(message) {
        const successDiv = document.getElementById('success-message');
        successDiv.textContent = message;
        successDiv.style.display = 'block';
        
        const errorDiv = document.getElementById('error-message');
        errorDiv.style.display = 'none';
    }

    hideError() {
        const errorDiv = document.getElementById('error-message');
        const successDiv = document.getElementById('success-message');
        if (errorDiv) errorDiv.style.display = 'none';
        if (successDiv) successDiv.style.display = 'none';
    }

    highlightField(fieldId, isError) {
        const field = document.getElementById(fieldId);
        if (field) {
            if (isError) {
                field.style.borderColor = '#e74c3c';
                field.style.boxShadow = '0 0 0 2px rgba(231, 76, 60, 0.2)';
            } else {
                field.style.borderColor = '#ddd';
                field.style.boxShadow = '';
            }
        }
    }

    clearAllFieldHighlights() {
        const fieldIds = [
            'project-name', 'project-directory',
            'admin-username', 'admin-email',
            'admin-password', 'admin-password-confirm'
        ];

        fieldIds.forEach(fieldId => {
            this.highlightField(fieldId, false);
        });
    }

    showFieldError(fieldId, message) {
        this.showError(message);
        this.highlightField(fieldId, true);

        // Focus the problematic field
        const field = document.getElementById(fieldId);
        if (field) {
            field.focus();
        }
    }
}

// Initialize the setup wizard when the page loads
console.log('📄 Setting up DOMContentLoaded listener...');

document.addEventListener('DOMContentLoaded', () => {
    console.log('🎯 DOMContentLoaded event fired');
    try {
        console.log('🔧 Creating QMSSetupWizard instance...');
        const wizard = new QMSSetupWizard();
        console.log('✅ QMSSetupWizard instance created successfully');

        // Make wizard globally accessible for debugging
        window.qmsWizard = wizard;
        console.log('🌐 QMS Wizard available globally as window.qmsWizard');
    } catch (error) {
        console.error('❌ Failed to create QMSSetupWizard:', error);
        console.error('Stack trace:', error.stack);

        // Show error to user
        const errorDiv = document.getElementById('error-message');
        if (errorDiv) {
            errorDiv.textContent = 'Failed to initialize setup wizard: ' + error.message;
            errorDiv.style.display = 'block';
        }
    }
});

console.log('📝 Setup JavaScript loaded completely');
