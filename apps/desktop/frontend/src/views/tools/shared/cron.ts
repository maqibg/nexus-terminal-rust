export type ParsedCronField = { wildcard: boolean; values: Set<number> };

type ParseCronFieldInput = {
  raw: string;
  min: number;
  max: number;
  mapValue?: (n: number) => number;
};

type ParsedCron = {
  minute: ParsedCronField;
  hour: ParsedCronField;
  dayOfMonth: ParsedCronField;
  month: ParsedCronField;
  dayOfWeek: ParsedCronField;
};

const MINUTE_MS = 60_000;
const MAX_ITERATIONS = 2_000_000;

function normalizeCronField(raw: string) {
  return raw.trim();
}

function isCronWildcard(field: string) {
  return field === '*' || field === '?';
}

function parseCronStep(stepPart: string | undefined) {
  if (!stepPart) {
    return 1;
  }
  const step = Number.parseInt(stepPart, 10);
  if (!Number.isFinite(step) || step <= 0) {
    throw new Error(`cron step 非法：${stepPart}`);
  }
  return step;
}

function mapAndValidateCronValue(options: {
  value: number;
  min: number;
  max: number;
  mapValue?: (n: number) => number;
}) {
  const mapped = options.mapValue ? options.mapValue(options.value) : options.value;
  if (mapped < options.min || mapped > options.max) {
    throw new Error(`cron 值超出范围：${options.value}（期望 ${options.min}-${options.max}）`);
  }
  return mapped;
}

function pushCronRange(options: {
  start: number;
  end: number;
  step: number;
  values: Set<number>;
  min: number;
  max: number;
  mapValue?: (n: number) => number;
}) {
  if (options.step <= 0) {
    throw new Error(`cron step 非法：${options.step}`);
  }
  for (let value = options.start; value <= options.end; value += options.step) {
    const mapped = mapAndValidateCronValue({
      value,
      min: options.min,
      max: options.max,
      mapValue: options.mapValue,
    });
    options.values.add(mapped);
  }
}

function parseCronItem(options: {
  item: string;
  values: Set<number>;
  min: number;
  max: number;
  mapValue?: (n: number) => number;
}) {
  const [rangePart, stepPart] = options.item.split('/');
  const step = parseCronStep(stepPart);

  if (rangePart === '*') {
    pushCronRange({
      start: options.min,
      end: options.max,
      step,
      values: options.values,
      min: options.min,
      max: options.max,
      mapValue: options.mapValue,
    });
    return;
  }

  const rangeMatch = rangePart.match(/^(\d+)-(\d+)$/);
  if (rangeMatch) {
    const start = Number.parseInt(rangeMatch[1], 10);
    const end = Number.parseInt(rangeMatch[2], 10);
    if (start > end) {
      throw new Error(`cron range 非法：${rangePart}`);
    }
    pushCronRange({
      start,
      end,
      step,
      values: options.values,
      min: options.min,
      max: options.max,
      mapValue: options.mapValue,
    });
    return;
  }

  const single = Number.parseInt(rangePart, 10);
  if (!Number.isFinite(single)) {
    throw new Error(`cron 值非法：${options.item}`);
  }
  const mapped = mapAndValidateCronValue({
    value: single,
    min: options.min,
    max: options.max,
    mapValue: options.mapValue,
  });
  options.values.add(mapped);
}

function parseCronField(input: ParseCronFieldInput): ParsedCronField {
  const field = normalizeCronField(input.raw);
  if (isCronWildcard(field)) {
    return { wildcard: true, values: new Set<number>() };
  }

  const values = new Set<number>();
  const items = field
    .split(',')
    .map(s => s.trim())
    .filter(Boolean);
  if (items.length === 0) {
    throw new Error(`cron 字段为空：${input.raw}`);
  }

  for (const item of items) {
    parseCronItem({ item, values, ...input });
  }

  return { wildcard: false, values };
}

function parseCron(expression: string): ParsedCron {
  const parts = expression.trim().split(/\s+/);
  if (parts.length !== 5) {
    throw new Error('仅支持 5 段 cron：分 时 日 月 周');
  }

  return {
    minute: parseCronField({ raw: parts[0], min: 0, max: 59 }),
    hour: parseCronField({ raw: parts[1], min: 0, max: 23 }),
    dayOfMonth: parseCronField({ raw: parts[2], min: 1, max: 31 }),
    month: parseCronField({ raw: parts[3], min: 1, max: 12 }),
    dayOfWeek: parseCronField({ raw: parts[4], min: 0, max: 7, mapValue: n => (n === 7 ? 0 : n) }),
  };
}

function cronFieldMatch(field: ParsedCronField, value: number) {
  return field.wildcard || field.values.has(value);
}

function cronMatches(options: { spec: ParsedCron; date: Date }) {
  if (!cronFieldMatch(options.spec.minute, options.date.getMinutes())) return false;
  if (!cronFieldMatch(options.spec.hour, options.date.getHours())) return false;
  if (!cronFieldMatch(options.spec.month, options.date.getMonth() + 1)) return false;

  const domOk = cronFieldMatch(options.spec.dayOfMonth, options.date.getDate());
  const dowOk = cronFieldMatch(options.spec.dayOfWeek, options.date.getDay());

  if (!options.spec.dayOfMonth.wildcard && !options.spec.dayOfWeek.wildcard) {
    return domOk || dowOk;
  }
  return domOk && dowOk;
}

export function previewCronDates(input: { expression: string; count: number; from?: Date }) {
  const spec = parseCron(input.expression);
  const results: Date[] = [];

  let current = input.from ? new Date(input.from.getTime()) : new Date();
  current.setSeconds(0, 0);
  current = new Date(current.getTime() + MINUTE_MS);

  for (let i = 0; results.length < input.count && i < MAX_ITERATIONS; i++) {
    if (cronMatches({ spec, date: current })) {
      results.push(current);
    }
    current = new Date(current.getTime() + MINUTE_MS);
  }

  return results;
}
