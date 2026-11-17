import { memo, useEffect, useRef } from "react";
import * as THREE from "three";

import { useSimulationStore } from "../store/simulationStore";
import { PerformanceTracker } from "../utils/PerformanceTracker";
import { SnapshotInterpolator } from "../utils/SnapshotInterpolator";

import type { SimulationSnapshot } from "../types/simulation";

const MAX_ORGANISMS = 100_000;
const MAX_FOOD = 100_000;

function EcosystemViewport() {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = containerRef.current;

    if (!container) return;

    // scene and camera

    const scene = new THREE.Scene();

    scene.background = new THREE.Color("#202820");

    const camera = new THREE.OrthographicCamera(0, 1600, 0, 900, 0.1, 100);

    camera.position.z = 10;

    const renderer = new THREE.WebGLRenderer({
      antialias: true,
      powerPreference: "high-performance",
    });

    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));

    container.appendChild(renderer.domElement);

    // organism mesh

    const organismGeometry = new THREE.CircleGeometry(1, 16);

    const organismMaterial = new THREE.MeshBasicMaterial({
      color: 0xffffff,
      side: THREE.DoubleSide,
    });

    const organisms = new THREE.InstancedMesh(
      organismGeometry,
      organismMaterial,
      MAX_ORGANISMS,
    );

    organisms.count = 0;
    organisms.frustumCulled = false;

    organisms.instanceMatrix.setUsage(THREE.DynamicDrawUsage);

    // reuse the instance color buffer

    organisms.setColorAt(0, new THREE.Color(0xffffff));

    organisms.instanceColor?.setUsage(THREE.DynamicDrawUsage);

    scene.add(organisms);

    // food mesh

    const foodGeometry = new THREE.CircleGeometry(1, 8);

    const foodMaterial = new THREE.MeshBasicMaterial({
      color: "#a3e635",
      transparent: true,
      opacity: 0.8,
      side: THREE.DoubleSide,
    });

    const food = new THREE.InstancedMesh(foodGeometry, foodMaterial, MAX_FOOD);

    food.count = 0;
    food.frustumCulled = false;

    food.instanceMatrix.setUsage(THREE.DynamicDrawUsage);

    scene.add(food);

    const zoneGeometry = new THREE.CircleGeometry(1, 64);
    const resourceMaterial = new THREE.MeshBasicMaterial({
      color: "#779b58",
      transparent: true,
      opacity: 0.16,
      depthWrite: false,
      side: THREE.DoubleSide,
    });
    const hazardMaterial = new THREE.MeshBasicMaterial({
      color: "#d97958",
      transparent: true,
      opacity: 0.24,
      depthWrite: false,
      side: THREE.DoubleSide,
    });
    const zones = [
      new THREE.Mesh(zoneGeometry, resourceMaterial),
      new THREE.Mesh(zoneGeometry, resourceMaterial),
      new THREE.Mesh(zoneGeometry, hazardMaterial),
    ];
    zones.forEach((zone) => {
      zone.visible = false;
      scene.add(zone);
    });

    function updateZones(snapshot: SimulationSnapshot) {
      const radius = Math.min(worldWidth, worldHeight);
      zones.forEach((zone, i) => {
        zone.visible =
          i === 2
            ? !!snapshot.habitat?.hazard_zones
            : !!snapshot.habitat?.resource_zones;
        zone.position.set(
          worldWidth * [0.25, 0.75, 0.5][i],
          worldHeight * 0.5,
          -2,
        );
        const r = radius * (i === 2 ? 0.12 : 0.15);
        zone.scale.set(r, r, 1);
      });
    }

    // rendering utilities

    const dummy = new THREE.Object3D();
    const color = new THREE.Color();

    const interpolator = new SnapshotInterpolator();
    const performanceTracker = new PerformanceTracker();

    let worldWidth = 1600;
    let worldHeight = 900;

    let lastSnapshot: SimulationSnapshot | null = null;

    let renderedSnapshot: SimulationSnapshot | null = null;

    let lastRenderedTick = -1;

    let lastMovementAt = 0;

    let needsRender = true;

    const previousOrganismIds = new Float64Array(MAX_ORGANISMS);

    const previousOrganismHues = new Float64Array(MAX_ORGANISMS);

    previousOrganismIds.fill(-1);
    previousOrganismHues.fill(-1);

    const previousFoodIds = new Float64Array(MAX_FOOD);

    const previousFoodX = new Float64Array(MAX_FOOD);

    const previousFoodY = new Float64Array(MAX_FOOD);

    previousFoodIds.fill(-1);

    // partial gpu buffer updates

    function uploadActiveRange(
      attribute: THREE.BufferAttribute,
      instanceCount: number,
      componentsPerInstance: number,
    ) {
      attribute.clearUpdateRanges();

      if (instanceCount > 0) {
        attribute.addUpdateRange(0, instanceCount * componentsPerInstance);

        attribute.needsUpdate = true;
      }
    }

    // resize handling

    function resize() {
      if (!container) return;

      const width = Math.max(1, container.clientWidth);

      const height = Math.max(1, container.clientHeight);

      const screenAspect = width / height;
      const worldAspect = worldWidth / worldHeight;

      let viewWidth = worldWidth;
      let viewHeight = worldHeight;

      if (screenAspect > worldAspect) {
        viewWidth = worldHeight * screenAspect;
      } else {
        viewHeight = worldWidth / screenAspect;
      }

      camera.left = (worldWidth - viewWidth) / 2;
      camera.right = camera.left + viewWidth;

      camera.top = (worldHeight - viewHeight) / 2;
      camera.bottom = camera.top + viewHeight;

      camera.updateProjectionMatrix();

      renderer.setSize(width, height);

      needsRender = true;
    }

    const observer = new ResizeObserver(resize);

    observer.observe(container);

    resize();

    // organism colors

    function updateOrganismColors(snapshot: SimulationSnapshot) {
      const organismCount = Math.min(
        snapshot.world.organisms.length,
        MAX_ORGANISMS,
      );

      let colorsChanged = false;

      for (let i = 0; i < organismCount; i++) {
        const organism = snapshot.world.organisms[i];

        const hue = snapshot.habitat?.predation
          ? organism.species === "predator"
            ? -2
            : -3
          : organism.genome.hue;

        if (
          previousOrganismIds[i] === organism.id &&
          previousOrganismHues[i] === hue
        ) {
          continue;
        }

        previousOrganismIds[i] = organism.id;
        previousOrganismHues[i] = hue;

        if (hue === -2) color.set("#d97958");
        else if (hue === -3) color.set("#8ab778");
        else color.setHSL(hue / 360, 0.85, 0.6);

        organisms.setColorAt(i, color);

        colorsChanged = true;
      }

      if (colorsChanged && organisms.instanceColor) {
        uploadActiveRange(organisms.instanceColor, organismCount, 3);
      }
    }

    // interpolated organism positions

    function updateOrganismPositions(
      snapshot: SimulationSnapshot,
      now: number,
    ) {
      const organismCount = Math.min(
        snapshot.world.organisms.length,
        MAX_ORGANISMS,
      );

      organisms.count = organismCount;

      for (let i = 0; i < organismCount; i++) {
        const organism = snapshot.world.organisms[i];

        const position = interpolator.getPosition(organism, now);

        dummy.position.set(position.x, position.y, 0);

        dummy.scale.set(organism.genome.size, organism.genome.size, 1);

        dummy.updateMatrix();

        organisms.setMatrixAt(i, dummy.matrix);
      }

      uploadActiveRange(organisms.instanceMatrix, organismCount, 16);

      renderedSnapshot = snapshot;
    }

    // food positions

    function updateFood(snapshot: SimulationSnapshot) {
      const foodCount = Math.min(snapshot.world.food.length, MAX_FOOD);

      let foodChanged = food.count !== foodCount;

      food.count = foodCount;

      for (let i = 0; i < foodCount; i++) {
        const resource = snapshot.world.food[i];

        if (
          previousFoodIds[i] === resource.id &&
          previousFoodX[i] === resource.x &&
          previousFoodY[i] === resource.y
        ) {
          continue;
        }

        previousFoodIds[i] = resource.id;
        previousFoodX[i] = resource.x;
        previousFoodY[i] = resource.y;

        dummy.position.set(resource.x, resource.y, -1);

        dummy.scale.set(2, 2, 1);

        dummy.updateMatrix();

        food.setMatrixAt(i, dummy.matrix);

        foodChanged = true;
      }

      if (foodChanged) {
        uploadActiveRange(food.instanceMatrix, foodCount, 16);
      }
    }

    // receive simulation snapshots

    function receiveSnapshot(snapshot: SimulationSnapshot, now: number) {
      performanceTracker.recordSnapshot();
      const world = snapshot.world;

      if (world.width !== worldWidth || world.height !== worldHeight) {
        worldWidth = world.width;
        worldHeight = world.height;

        resize();
      }

      const tickChanged = snapshot.status.tick !== lastRenderedTick;

      interpolator.push(snapshot, now);

      if (tickChanged || !lastSnapshot) {
        lastMovementAt = now;
      }

      lastRenderedTick = snapshot.status.tick;

      updateOrganismColors(snapshot);

      updateFood(snapshot);
      updateZones(snapshot);

      lastSnapshot = snapshot;

      needsRender = true;
    }

    // animation loop

    let animationId: number;

    function animate(now: number) {
      const snapshot = useSimulationStore.getState().snapshot;

      if (snapshot && snapshot !== lastSnapshot) {
        receiveSnapshot(snapshot, now);
      }

      const currentSnapshot = interpolator.getSnapshot();

      const interpolationActive = now - lastMovementAt < 200;

      let rendered = false;

      if (currentSnapshot && (interpolationActive || needsRender)) {
        updateOrganismPositions(currentSnapshot, now);

        renderer.render(scene, camera);

        rendered = true;

        needsRender = false;
      } else if (needsRender) {
        renderer.render(scene, camera);

        rendered = true;

        needsRender = false;
      }

      performanceTracker.recordFrame(now, renderer, rendered);

      animationId = requestAnimationFrame(animate);
    }

    animationId = requestAnimationFrame(animate);

    // organism selection

    const raycaster = new THREE.Raycaster();

    const pointer = new THREE.Vector2();

    function handleClick(event: MouseEvent) {
      const bounds = renderer.domElement.getBoundingClientRect();

      pointer.x = ((event.clientX - bounds.left) / bounds.width) * 2 - 1;

      pointer.y = -((event.clientY - bounds.top) / bounds.height) * 2 + 1;

      raycaster.setFromCamera(pointer, camera);

      organisms.computeBoundingSphere();

      const hits = raycaster.intersectObject(organisms);

      const instanceId = hits[0]?.instanceId;

      // select from the rendered snapshot

      const organism =
        instanceId !== undefined
          ? renderedSnapshot?.world.organisms[instanceId]
          : undefined;

      useSimulationStore.getState().selectOrganism(organism?.id ?? null);
    }

    renderer.domElement.addEventListener("click", handleClick);

    // cleanup

    return () => {
      cancelAnimationFrame(animationId);

      observer.disconnect();

      renderer.domElement.removeEventListener("click", handleClick);

      organisms.dispose();
      food.dispose();

      organismGeometry.dispose();
      organismMaterial.dispose();

      foodGeometry.dispose();
      foodMaterial.dispose();

      zoneGeometry.dispose();
      resourceMaterial.dispose();
      hazardMaterial.dispose();
      renderer.dispose();

      renderer.domElement.remove();
    };
  }, []);

  return <div ref={containerRef} className="ecosystem-viewport" />;
}
export default memo(EcosystemViewport);
