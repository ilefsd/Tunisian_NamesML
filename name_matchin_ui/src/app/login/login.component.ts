import { Component, OnInit } from '@angular/core';
import { FormBuilder, FormGroup, Validators } from '@angular/forms';
import { Router } from '@angular/router';
import { AuthService } from '../services/auth.service';

@Component({
  selector: 'app-login',
  standalone: false,
  templateUrl: './login.component.html',
  styleUrl: './login.component.css'
})
export class LoginComponent implements OnInit {
  loginForm: FormGroup;
  showPassword = false;
  isLoading = false;
  errorMessage = '';

  constructor(
    private fb: FormBuilder,
    private authService: AuthService,
    private router: Router
  ) {
    this.loginForm = this.fb.group({
      email: ['', [Validators.required, Validators.email]],
      password: ['', [Validators.required]],
      rememberMe: [false]
    });
  }

  ngOnInit(): void {
    // Check if user is already logged in
    if (this.authService.isLoggedIn()) {
      this.router.navigate(['/']);
    }
  }

  /**
   * Toggle password visibility
   */
  togglePasswordVisibility(): void {
    this.showPassword = !this.showPassword;
  }

  /**
   * Handle form submission
   */
  onSubmit(): void {
    if (this.loginForm.valid && !this.isLoading) {
      this.isLoading = true;
      this.errorMessage = '';

      const loginData = {
        email: this.loginForm.get('email')?.value,
        password: this.loginForm.get('password')?.value,
        rememberMe: this.loginForm.get('rememberMe')?.value
      };

      this.authService.login(loginData).subscribe({
        next: (response) => {
          this.isLoading = false;
          // Handle successful login
          console.log('Login successful', response);
          this.router.navigate(['/identity-match']);
        },
        error: (error) => {
          this.isLoading = false;
          // Handle login error
          console.error('Login error:', error);

          // Set user-friendly error message
          if (error.status === 401) {
            this.errorMessage = 'البريد الإلكتروني أو كلمة المرور غير صحيحة';
          } else if (error.status === 0) {
            this.errorMessage = 'خطأ في الاتصال بالخادم. يرجى المحاولة مرة أخرى';
          } else {
            this.errorMessage = 'حدث خطأ غير متوقع. يرجى المحاولة مرة أخرى';
          }
        }
      });
    } else {
      // Mark all fields as touched to show validation errors
      this.markFormGroupTouched(this.loginForm);
    }
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
   * Navigate to register page
   */
  navigateToRegister(): void {
    this.router.navigate(['/register']);
  }

  /**
   * Handle forgot password
   */
  onForgotPassword(): void {
    // Implement forgot password logic
    console.log('Forgot password clicked');
    // You can navigate to forgot password page or show a modal
    // this.router.navigate(['/forgot-password']);
  }

  /**
   * Get error message for a specific form control
   */
  getErrorMessage(controlName: string): string {
    const control = this.loginForm.get(controlName);

    if (control?.errors && control.touched) {
      if (control.errors['required']) {
        switch (controlName) {
          case 'email':
            return 'البريد الإلكتروني مطلوب';
          case 'password':
            return 'كلمة المرور مطلوبة';
          default:
            return 'هذا الحقل مطلوب';
        }
      }

      if (control.errors['email']) {
        return 'يرجى إدخال بريد إلكتروني صحيح';
      }
    }

    return '';
  }

  /**
   * Check if a form control has errors and is touched
   */
  hasError(controlName: string): boolean {
    const control = this.loginForm.get(controlName);
    return !!(control?.errors && control.touched);
  }

  /**
   * Clear error message
   */
  clearError(): void {
    this.errorMessage = '';
  }
}
