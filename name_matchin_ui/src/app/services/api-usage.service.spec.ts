import { TestBed } from '@angular/core/testing';

import { ApiUsageService } from './api-usage.service';

describe('ApiUsageService', () => {
  let service: ApiUsageService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(ApiUsageService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
