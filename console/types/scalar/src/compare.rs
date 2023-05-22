// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the snarkVM library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;

impl<E: Environment> Ord for Scalar<E> {
    /// Returns the lexicographic ordering of `self` and `other`.
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_bigint().cmp(&other.to_bigint())
    }
}

impl<E: Environment> PartialOrd for Scalar<E> {
    /// Returns the lexicographic ordering of `self` and `other`.
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
