import { Component } from '@angular/core';
import { FormBuilder, FormGroup, Validators } from '@angular/forms';
import { IdentityMatchService, InputIdentity, MatchResult } from '../services/identity-match.service';
import { MatDialog } from '@angular/material/dialog';
import { FamilyTreeComponent } from '../family-tree/family-tree.component';
import { ApiUsagePopupComponent } from '../api-usage-popup/api-usage-popup.component';
import { ApiUsageService } from '../services/api-usage.service';
import { AuthService } from '../services/auth.service';
import { Router } from '@angular/router';

@Component({
  selector: 'app-identity-match',
  standalone: false,
  templateUrl: './identity-match.component.html',
  styleUrls: ['./identity-match.component.css']
})
export class IdentityMatchComponent {

  form: FormGroup;
  results: MatchResult[] = [];
  loading = false;
  error: string | null = null;

  constructor(
    private fb: FormBuilder,
    private matchSvc: IdentityMatchService,
    public dialog: MatDialog,
    private authService: AuthService,
    private router: Router,
    private apiUsageService: ApiUsageService
  ) {
    this.form = this.fb.group({
      first_name:       ['', Validators.required],
      last_name:        ['', Validators.required],
      father_name:      [''],
      grandfather_name: [''],
      mother_last_name: [''],
      mother_name:      [''],
      dob_day:          [''],
      dob_month:        [''],
      dob_year:         [''],
      sex:              [1],
      place_of_birth:   ['']
    });
  }

  logout(): void {
    this.authService.logout();
    this.router.navigate(['/login']);
  }

  openFamilyTreeModal(matchResult: MatchResult): void {
    const dialogRef = this.dialog.open(FamilyTreeComponent, {
      width: '80vw', // Consider making this responsive or using CSS classes
      maxWidth: '95vw',
      maxHeight: '90vh',
      data: { identity: matchResult.matched_identity },
      panelClass: 'family-tree-dialog-container' // For custom global styling if needed
    });

    dialogRef.afterClosed().subscribe((dialogResult: any) => {
      console.log('The dialog was closed', dialogResult);
      // You can add logic here if needed after the dialog closes
    });
    // console.log("Opening modal for:", matchResult.matched_identity); // Placeholder action removed
  }

  onSubmit() {
    if (this.form.invalid) {
      return;
    }

    const v = this.form.value;
    const input: InputIdentity = {
      first_name:       v.first_name,
      last_name:        v.last_name,
      father_name:      v.father_name,
      grandfather_name: v.grandfather_name,
      mother_last_name: v.mother_last_name,
      mother_name:      v.mother_name,
      dob:              (v.dob_day && v.dob_month && v.dob_year)
        ? [ +v.dob_day, +v.dob_month, +v.dob_year ]
        : null,
      sex:              +v.sex,
      place_of_birth:   v.place_of_birth
    };

    this.loading = true;
    this.error = null;
    this.matchSvc.matchIdentity(input).subscribe({
      next: (rs: MatchResult[]) => {
        this.results = rs;
        this.loading = false;
      },
      error: (err: any) => {
        this.loading = false;
        // If the backend returns 400 with { message: "لا يوجد تطابق بسبب اختلاف الجنس" }
        if (err.status === 400 && err.error && err.error.message) {
          this.error = err.error.message;
        } else {
          // Generic fallback
          this.error = 'حدث خطأ أثناء المطابقة. يُرجى المحاولة مجددًا.';
        }
      }
    });
  }

  openPopup(): void {
    const userId = this.authService.getUserId();
    if (userId) {
      this.apiUsageService.getApiUsage(userId).subscribe(apiUsage => {
        this.dialog.open(ApiUsagePopupComponent, {
          width: '600px',
          data: { apiUsage }
        });
      });
    }
  }
}
