import { Injectable } from '@angular/core';
import neo4j, { Driver, Session } from 'neo4j-driver';
import { FamilyPath } from '../models/family-path/family-path.module';
import { environment } from '../../environments/environment';

@Injectable({ providedIn: 'root' })
export class Neo4jService {
  private driver: Driver;

  constructor() {
    this.driver = neo4j.driver(
      environment.neo4j.boltUrl,
      neo4j.auth.basic(environment.neo4j.user, environment.neo4j.pass)
    );
  }

  async fetchBestFamilyTrees(
    childName: string,
    fatherName: string,
    motherName: string
  ): Promise<FamilyPath[]> {
    const session: Session = this.driver.session();
    const cypher = `
      WITH
        $childName  AS childName,
        $fatherName AS fatherName,
        $motherName AS motherName

      MATCH (c:Person {name: childName})
      OPTIONAL MATCH (c)-[:CHILD_OF]->(f:Person) WHERE f.gender = 'ذكر'
      OPTIONAL MATCH (c)-[:CHILD_OF]->(m:Person) WHERE m.gender = 'أنثى'
      WITH c, f, m,
        (CASE WHEN f.name = fatherName THEN 1 ELSE 0 END
       + CASE WHEN m.name = motherName THEN 1 ELSE 0 END) AS score
      ORDER BY score DESC
      LIMIT 3
      OPTIONAL MATCH (c)-[:CHILD_OF]->(parent:Person)
      OPTIONAL MATCH (c)-[:SIBLING_WITH]-(sib:Person)
      OPTIONAL MATCH (childRel:Person)-[:CHILD_OF]->(c)
      OPTIONAL MATCH (c)-[:MARRIED_TO]-(sp:Person)
      WITH
        c, score,
        collect(DISTINCT parent)[0..2]   AS parents,
        collect(DISTINCT sib)[0..3]      AS siblings,
        collect(DISTINCT childRel)[0..3] AS children,
        collect(DISTINCT sp)[0..1]       AS spouses
      UNWIND parents + siblings + children + spouses AS relative
      MATCH path = (c)-[:CHILD_OF|SIBLING_WITH|MARRIED_TO]-(relative)
      RETURN
        c.name    AS candidate,
        score,
        path
      ORDER BY candidate, score DESC;
    `;
    const result = await session.run(cypher, { childName, fatherName, motherName });
    await session.close();

    return result.records.map(rec => ({
      candidate: rec.get('candidate'),
      score: rec.get('score').toNumber(),
      // rec.get('path') is a Path object with .segments, each has .start, .end, .relationship
      path: rec.get('path')
    }));
  }
}
