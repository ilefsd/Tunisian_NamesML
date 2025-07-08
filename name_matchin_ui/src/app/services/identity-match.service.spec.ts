import { TestBed } from '@angular/core/testing';

import { IdentityMatchService } from './identity-match.service';

describe('IdentityMatchService', () => {
  let service: IdentityMatchService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(IdentityMatchService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
