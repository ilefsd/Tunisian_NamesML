import { ComponentFixture, TestBed } from '@angular/core/testing';

import { IdentityMatchComponent } from './identity-match.component';

describe('IdentityMatchComponent', () => {
  let component: IdentityMatchComponent;
  let fixture: ComponentFixture<IdentityMatchComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [IdentityMatchComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(IdentityMatchComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
