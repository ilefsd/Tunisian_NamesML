import { Component, OnInit } from '@angular/core';
import { FormBuilder, FormGroup, Validators, AbstractControl, ValidationErrors } from '@angular/forms';
import { Router } from '@angular/router';
import { AuthService } from '../services/auth.service';

@Component({
  selector: 'app-register',
  standalone: false,
  templateUrl: './register.component.html',
  styleUrl: './register.component.css'
})
export class RegisterComponent implements OnInit {
  registerForm: FormGroup;

  // UI State Properties
  isLoading = false;
  showPassword = false;
  showConfirmPassword = false;

  // Message Properties
  successMessage = '';
  errorMessage = '';

  constructor(
    private fb: FormBuilder,
    private authService: AuthService,
    private router: Router
  ) {
    this.registerForm = this.fb.group({
      email: ['', [Validators.required, Validators.email]],
      password: ['', [Validators.required, Validators.minLength(8)]],
      confirmPassword: ['', [Validators.required]],
      acceptTerms: [false, [Validators.requiredTrue]]
    }, {
      validators: [this.passwordMatchValidator]
    });
  }

  ngOnInit(): void {
    // Component initialization logic can be added here
  }

  /**
   * Custom validator to check if password and confirmPassword match
   */
  passwordMatchValidator(control: AbstractControl): ValidationErrors | null {
    const password = control.get('password');
    const confirmPassword = control.get('confirmPassword');

    if (!password || !confirmPassword) {
      return null;
    }

    if (password.value !== confirmPassword.value) {
      return { passwordMismatch: true };
    }

    return null;
  }

  /**
   * Toggle password visibility
   */
  togglePasswordVisibility(): void {
    this.showPassword = !this.showPassword;
  }

  /**
   * Toggle confirm password visibility
   */
  toggleConfirmPasswordVisibility(): void {
    this.showConfirmPassword = !this.showConfirmPassword;
  }

  /**
   * Clear error message
   */
  clearError(): void {
    this.errorMessage = '';
  }

  /**
   * Clear success message
   */
  clearSuccess(): void {
    this.successMessage = '';
  }

  /**
   * Handle terms and conditions link click
   */
  openTerms(event: Event): void {
    event.preventDefault();
    // Implement terms and conditions modal or navigation
    console.log('Opening terms and conditions...');
    // You can implement a modal service or navigate to a terms page
    // this.modalService.openTermsModal();
    // or
    // this.router.navigate(['/terms']);
  }

  /**
   * Handle privacy policy link click
   */
  openPrivacy(event: Event): void {
    event.preventDefault();
    // Implement privacy policy modal or navigation
    console.log('Opening privacy policy...');
    // You can implement a modal service or navigate to a privacy page
    // this.modalService.openPrivacyModal();
    // or
    // this.router.navigate(['/privacy']);
  }

  /**
   * Handle form submission
   */
  onSubmit(): void {
    // Clear previous messages
    this.clearError();
    this.clearSuccess();

    // Check if form is valid
    if (this.registerForm.invalid) {
      // Mark all fields as touched to show validation errors
      this.markFormGroupTouched(this.registerForm);
      this.errorMessage = 'يرجى تصحيح الأخطاء في النموذج';
      return;
    }

    // Set loading state
    this.isLoading = true;

    // Prepare registration data (exclude confirmPassword from API call)
    const registrationData = {
      email: this.registerForm.get('email')?.value,
      password: this.registerForm.get('password')?.value
    };

    // Call authentication service
    this.authService.register(registrationData).subscribe({
      next: (response) => {
        this.isLoading = false;
        this.successMessage = 'تم إنشاء الحساب بنجاح! سيتم توجيهك إلى صفحة تسجيل الدخول...';

        // Navigate to login after a short delay to show success message
        setTimeout(() => {
          this.router.navigate(['/login'], {
            queryParams: {
              message: 'تم إنشاء الحساب بنجاح. يمكنك الآن تسجيل الدخول.'
            }
          });
        }, 2000);
      },
      error: (error) => {
        this.isLoading = false;

        // Handle different types of errors
        if (error.status === 409) {
          this.errorMessage = 'البريد الإلكتروني مستخدم بالفعل';
        } else if (error.status === 400) {
          this.errorMessage = 'البيانات المدخلة غير صحيحة';
        } else if (error.status === 0) {
          this.errorMessage = 'خطأ في الاتصال بالخادم. يرجى المحاولة لاحقاً';
        } else {
          this.errorMessage = error.error?.message || 'حدث خطأ أثناء إنشاء الحساب. يرجى المحاولة مرة أخرى';
        }

        console.error('Registration error:', error);
      }
    });
  }

  /**
   * Mark all form controls as touched to trigger validation display
   */
  private markFormGroupTouched(formGroup: FormGroup): void {
    Object.keys(formGroup.controls).forEach(key => {
      const control = formGroup.get(key);
      control?.markAsTouched();

      if (control instanceof FormGroup) {
        this.markFormGroupTouched(control);
      }
    });
  }

  /**
   * Get form control for easier access in template
   */
  getFormControl(controlName: string): AbstractControl | null {
    return this.registerForm.get(controlName);
  }

  /**
   * Check if a form control has a specific error
   */
  hasError(controlName: string, errorType: string): boolean {
    const control = this.getFormControl(controlName);
    return !!(control && control.errors && control.errors[errorType] && control.touched);
  }

  /**
   * Get error message for a specific control
   */
  getErrorMessage(controlName: string): string {
    const control = this.getFormControl(controlName);

    if (!control || !control.errors || !control.touched) {
      return '';
    }

    const errors = control.errors;

    switch (controlName) {
      case 'email':
        if (errors['required']) return 'البريد الإلكتروني مطلوب';
        if (errors['email']) return 'يرجى إدخال بريد إلكتروني صحيح';
        break;

      case 'password':
        if (errors['required']) return 'كلمة المرور مطلوبة';
        if (errors['minlength']) return 'كلمة المرور يجب أن تكون 8 أحرف على الأقل';
        break;

      case 'confirmPassword':
        if (errors['required']) return 'تأكيد كلمة المرور مطلوب';
        break;

      case 'acceptTerms':
        if (errors['required']) return 'يجب الموافقة على الشروط والأحكام';
        break;
    }

    // Check for form-level errors
    if (this.registerForm.errors?.['passwordMismatch'] && controlName === 'confirmPassword') {
      return 'كلمات المرور غير متطابقة';
    }

    return 'خطأ في البيانات المدخلة';
  }

  /**
   * Reset form to initial state
   */
  resetForm(): void {
    this.registerForm.reset();
    this.clearError();
    this.clearSuccess();
    this.showPassword = false;
    this.showConfirmPassword = false;
    this.isLoading = false;
  }

  /**
   * Check if form is ready for submission
   */
  get isFormReady(): boolean {
    return this.registerForm.valid && !this.isLoading;
  }

  /**
   * Get current form validation status
   */
  get formStatus(): string {
    if (this.registerForm.pending) return 'pending';
    if (this.registerForm.valid) return 'valid';
    if (this.registerForm.invalid) return 'invalid';
    return 'pristine';
  }
}
