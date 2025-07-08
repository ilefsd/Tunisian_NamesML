import { TestBed } from '@angular/core/testing';

import { Neo4jServiceTsService } from './neo4j.service.ts.service';

describe('Neo4jServiceTsService', () => {
  let service: Neo4jServiceTsService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(Neo4jServiceTsService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
