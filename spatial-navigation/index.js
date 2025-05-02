// Import our outputted wasm ES6 module
// Which, export default's, an initialization function
import init, * as wasm from "./pkg/spatial_navigation_wasm.js";

const runWasm = async () => {
  await init();
  // Initialize the WASM module
  wasm.init();
  console.log('WASM module initialized');
  // Direction constants to match JavaScript spatial-navigation-polyfill
  const Direction = {
    left: wasm.Direction.Left,
    right: wasm.Direction.Right,
    up: wasm.Direction.Up,
    down: wasm.Direction.Down
  };

  /**
   * Convert a DOMRect to a WASM-compatible Rect
   * @param {DOMRect} domRect 
   * @returns {wasm.Rect}
   */
  function createWasmRect(domRect) {
    return new wasm.Rect(
      domRect.width,
      domRect.height,
      domRect.top,
      domRect.right,
      domRect.bottom,
      domRect.left
    );
  }

  /**
   * Convert a point object to WASM-compatible Point
   * @param {Object} point 
   * @returns {wasm.Point}
   */
  function createWasmPoint(point) {
    return new wasm.Point(point.x, point.y);
  }

  window.wasmNavigationHelper = {
    renderNavigableElements(numberOfElements) {
      return wasm.render_navigable_elements(numberOfElements);
    },
    /**
     * Check if one element is outside another element in a specific direction
     */
    isOutside(rect1, rect2, dir) {
      const wasmRect1 = createWasmRect(rect1);
      const wasmRect2 = createWasmRect(rect2);
      return wasm.is_outside(wasmRect1, wasmRect2, Direction[dir]);
    },

    /**
     * Check if one element is to the right of another
     */
    isRightSide(rect1, rect2) {
      const wasmRect1 = createWasmRect(rect1);
      const wasmRect2 = createWasmRect(rect2);
      return wasm.is_right_side(wasmRect1, wasmRect2);
    },

    /**
     * Check if one element is below another
     */
    isBelow(rect1, rect2) {
      const wasmRect1 = createWasmRect(rect1);
      const wasmRect2 = createWasmRect(rect2);
      return wasm.is_below(wasmRect1, wasmRect2);
    },

    /**
     * Check if elements are aligned in a direction
     */
    isAligned(rect1, rect2, dir) {
      const wasmRect1 = createWasmRect(rect1);
      const wasmRect2 = createWasmRect(rect2);
      return wasm.is_aligned(wasmRect1, wasmRect2, Direction[dir]);
    },

    /**
     * Check if one element is inside another
     */
    isInside(containerRect, childRect) {
      const wasmContainerRect = createWasmRect(containerRect);
      const wasmChildRect = createWasmRect(childRect);
      return wasm.is_inside(wasmContainerRect, wasmChildRect);
    },

    /**
     * Get distance from a point to an element
     */
    getDistanceFromPoint(point, element, dir) {
      const wasmPoint = createWasmPoint(point);
      const wasmRect = createWasmRect(element);
      return wasm.get_distance_from_point(wasmPoint, wasmRect, Direction[dir]);
    },

    /**
     * Get inner distance between rectangles
     */
    getInnerDistance(rect1, rect2, dir) {
      const wasmRect1 = createWasmRect(rect1);
      const wasmRect2 = createWasmRect(rect2);
      return wasm.get_inner_distance(wasmRect1, wasmRect2, Direction[dir]);
    },

    /**
     * Get euclidean distance between rectangles
     */
    getEuclideanDistance(rect1, rect2, dir) {
      const wasmRect1 = createWasmRect(rect1);
      const wasmRect2 = createWasmRect(rect2);
      return wasm.get_euclidean_distance(wasmRect1, wasmRect2, Direction[dir]);
    },

    /**
     * Get absolute distance between rectangles
     */
    getAbsoluteDistance(rect1, rect2, dir) {
      const wasmRect1 = createWasmRect(rect1);
      const wasmRect2 = createWasmRect(rect2);
      return wasm.get_absolute_distance(wasmRect1, wasmRect2, Direction[dir]);
    },

    /**
     * Get weighted distance between rectangles
     */
    getDistance(searchOrigin, candidateRect, dir) {
      const wasmSearchOrigin = createWasmRect(searchOrigin);
      const wasmCandidateRect = createWasmRect(candidateRect);
      return wasm.get_distance(wasmSearchOrigin, wasmCandidateRect, Direction[dir]);
    },
    getEntryAndExitPoints(dir, searchOrigin, candidateRect, startingPoint) {
      const wasmDir = Direction[dir];
      const wasmSearchOrigin = searchOrigin ? createWasmRect(searchOrigin) : null;
      const wasmCandidateRect = createWasmRect(candidateRect);
      if(startingPoint)
             wasmCandidateRect.set_starting_point(startingPoint.x, startingPoint.y);
      const result = wasm.get_entry_and_exit_points(wasmDir, wasmSearchOrigin, wasmCandidateRect);
      return {
        entryPoint: {
          x: result.entry_point.x,
          y: result.entry_point.y
        },
        exitPoint: {
          x: result.exit_point.x,
          y: result.exit_point.y
        }
      };
    }
  };
};
runWasm();