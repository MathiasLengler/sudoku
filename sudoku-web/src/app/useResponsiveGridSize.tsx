import * as React from "react";
import {useCallback, useLayoutEffect, useMemo, useState} from "react";

function getSize() {
  return {
    width: window.innerWidth,
    height: window.innerHeight
  };
}

function useWindowSize() {
  const [windowSize, setWindowSize] = useState(getSize);

  useLayoutEffect(() => {
    function handleResize() {
      setWindowSize(getSize());
    }

    window.addEventListener('resize', handleResize);
    return () => {
      window.removeEventListener('resize', handleResize);
    };
  }, []); // Empty array ensures that effect is only run on mount and unmount

  return windowSize;
}

export function useResponsiveGridSize(toolbarHeight: number, sideLength: TransportSudoku['sideLength']) {
  const windowSize = useWindowSize();

  return useMemo(() => {
    console.debug("toolbarHeight", toolbarHeight);
    console.debug("windowSize", windowSize);

    const gridAndSelectorHeight = (windowSize.height - (toolbarHeight));
    console.debug("gridAndSelectorHeight", gridAndSelectorHeight);

    const gridHeight = (sideLength * gridAndSelectorHeight) / (sideLength + 1);
    console.debug("gridHeight", gridHeight);

    const gridSize = Math.min(gridHeight, windowSize.width);
    console.debug("GridSize", gridSize);

    return gridSize;
  }, [sideLength, toolbarHeight, windowSize]);
}

export type ElementRef = ((element: (Element | null)) => void);

export function useClientHeight(): [number, ElementRef] {
  const [height, setHeight] = useState(0);
  const ref = useCallback((element: Element | null) => {
    if (element !== null) {
      const {height} = element.getBoundingClientRect();
      setHeight(height);
    }
  }, []);

  return [height, ref];
}