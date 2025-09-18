import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ApiUsagePopupComponent } from './api-usage-popup.component';

describe('ApiUsagePopupComponent', () => {
  let component: ApiUsagePopupComponent;
  let fixture: ComponentFixture<ApiUsagePopupComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ApiUsagePopupComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ApiUsagePopupComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
