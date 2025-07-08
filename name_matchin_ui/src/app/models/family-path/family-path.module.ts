// src/app/models/family-path.model.ts
import { Node as NeoNode, Relationship as NeoRel } from 'neo4j-driver';

export interface FamilyPath {
  candidate: string;
  score: number;
  path: {
    segments: Array<{
      start: NeoNode;
      relationship: NeoRel;
      end: NeoNode;
    }>
  }
}
