import { Component, Inject } from '@angular/core';
import { MAT_DIALOG_DATA } from '@angular/material/dialog';

@Component({
  selector: 'app-api-usage-popup',
  standalone: false,
  templateUrl: './api-usage-popup.component.html',
  styleUrls: ['./api-usage-popup.component.css']
})
export class ApiUsagePopupComponent {
  constructor(@Inject(MAT_DIALOG_DATA) public data: { apiUsage: any[] }) { }
}
