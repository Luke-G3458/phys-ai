export type Vec2 = {
  x: number;
  y: number;
};

export type Pose2 = Vec2 & {
  orientation: number;
};

export type Bounds = {
  width: number;
  height: number;
};

export type Wall = {
  start: Vec2;
  end: Vec2;
  thickness: number;
};

export type BodySnapshot = {
  id: number;
  pose: Pose2;
  length: number;
  width: number;
};

export type WorldSnapshot = {
  bounds: Bounds;
  walls: Wall[];
  bodies: BodySnapshot[];
};

export type MotorCommand = {
  left: number;
  right: number;
};

export type RobotSnapshot = {
  pose: Pose2;
  length: number;
  width: number;
  motorCommand: MotorCommand;
  lidar: number[];
  lidarMaximumRange: number;
  collided: boolean;
};

export type SimulationSnapshot = {
  world: WorldSnapshot;
  robot: RobotSnapshot;
  running: boolean;
  stepsPerSnapshot: number;
  fixedStepSeconds: number;
  elapsedSeconds: number;
  stepCount: number;
};

export type GenerateMapPayload = {
  width: number;
  height: number;
  wallCount: number;
  wallThickness: number;
  minimumWallLength: number;
  maximumWallLength: number;
};
