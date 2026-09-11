export function getArea(p0: [number, number], p1: [number, number], p2: [number, number]): number {
  return (
    p0[0] * p1[1] +
    p1[0] * p2[1] +
    p2[0] * p0[1] -
    p1[0] * p0[1] -
    p2[0] * p1[1] -
    p0[0] * p2[1]
  ) / 2;
}

export function getPolygonCenter(points: Array<[number, number]>): [number, number] {
  if (points.length < 2) return [0, 0];

  let sumX = 0;
  let sumY = 0;
  let sumArea = 0;
  const p0 = points[0]!;
  for (let i = 1; i < points.length - 1; i++) {
    const p1 = points[i]!;
    const p2 = points[i + 1]!;
    const area = getArea(p0, p1, p2);
    sumArea += area;
    sumX += (p0[0] + p1[0] + p2[0]) * area;
    sumY += (p0[1] + p1[1] + p2[1]) * area;
  }

  return [sumX / sumArea / 3, sumY / sumArea / 3];
}

export function isInPolygon(
  checkPoint: [number, number],
  polygonPoints: Array<[number, number]>,
): boolean {
  const pointCount = polygonPoints.length;
  if (pointCount === 0) return false;

  let counter = 0;
  let p1 = polygonPoints[0]!;

  for (let i = 1; i <= pointCount; i++) {
    const p2 = polygonPoints[i % pointCount]!;
    if (
      checkPoint[0] > Math.min(p1[0], p2[0]) &&
      checkPoint[0] <= Math.max(p1[0], p2[0])
    ) {
      if (checkPoint[1] <= Math.max(p1[1], p2[1])) {
        if (p1[0] !== p2[0]) {
          const xinters =
            ((checkPoint[0] - p1[0]) * (p2[1] - p1[1])) / (p2[0] - p1[0]) + p1[1];
          if (p1[1] === p2[1] || checkPoint[1] <= xinters) {
            counter++;
          }
        }
      }
    }
    p1 = p2;
  }

  return counter % 2 !== 0;
}
