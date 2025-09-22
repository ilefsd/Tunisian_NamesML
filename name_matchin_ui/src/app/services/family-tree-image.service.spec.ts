import { TestBed } from '@angular/core/testing';

import { FamilyTreeImageService } from './family-tree-image.service';

describe('FamilyTreeImageService', () => {
  let service: FamilyTreeImageService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(FamilyTreeImageService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
